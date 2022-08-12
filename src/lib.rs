use current_state::CurrentState;
use itertools::Itertools;
use log::{debug, info, warn};
use polynomial::Polynomial;
use rand::prelude::Rng;
use rand::{seq::IteratorRandom, thread_rng};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use threadpool::ThreadPool;

pub mod current_state;
pub mod polynomial;
pub mod polynomial_verifier;

fn generate_mutated_polynomials(
    base_polynomial: &Polynomial,
    mutated_polynomials_to_evaluate: usize,
) -> Vec<Polynomial> {
    info!(
        "Start generating mutated polynomials for {}",
        base_polynomial.to_string()
    );
    let mut mutated_polynomials = Vec::new();
    let mut rng = rand::thread_rng();
    for i in 1..base_polynomial.len() {
        let combinations_of_i = (0..base_polynomial.len()).combinations(i);
        for combination in combinations_of_i {
            for _ in 0..mutated_polynomials_to_evaluate {
                let mut polynomial = base_polynomial.clone();
                for j in combination.clone() {
                    if polynomial[j] > 0.0 {
                        let num_up = if base_polynomial[j] >= 1.0 {
                            0.0
                        } else {
                            rng.gen_range(0.0..(1.0 - base_polynomial[j]))
                        };
                        let num_down = rng.gen_range(0.0..base_polynomial[j]);
                        polynomial[j] -= num_down;
                        polynomial[j] += num_up;
                    } else if polynomial[j] == 0.0 {
                        polynomial[j] += rng.gen_range(0.0..1.0);
                    } else {
                        let num_down = if base_polynomial[j].abs() >= 1.0 {
                            0.0
                        } else {
                            rng.gen_range(0.0..(1.0 - base_polynomial[j].abs()))
                        };
                        let num_up = rng.gen_range(0.0..base_polynomial[j].abs());
                        polynomial[j] -= num_down;
                        polynomial[j] += num_up;
                    }
                }
                mutated_polynomials.push(polynomial);
            }
        }
    }
    info!(
        "Generated {} mutated polynomials",
        mutated_polynomials.len()
    );
    if mutated_polynomials.len() > mutated_polynomials_to_evaluate {
        let mut rng = thread_rng();
        let mut vec = Vec::new();
        for entry in mutated_polynomials
            .iter()
            .choose_multiple(&mut rng, mutated_polynomials_to_evaluate)
        {
            vec.push(entry.clone());
        }
        vec
    } else {
        mutated_polynomials
    }
}

pub fn initialize_current_state(
    base_polynomial: &Polynomial,
    mutated_polynomials_to_evaluate: usize,
) -> CurrentState {
    let mut current_state = CurrentState::new(base_polynomial.len(), 0);
    current_state.starting_mutated_polynomials =
        generate_mutated_polynomials(&base_polynomial, mutated_polynomials_to_evaluate);
    debug!("Generated mutated polynomials:");
    for poly in &current_state.starting_mutated_polynomials {
        debug!("{}", poly.to_string());
    }
    current_state
}

pub fn print_finished_combination(combination: &Vec<usize>) {
    let mut combo_string = String::new();
    for combo in combination {
        combo_string = format!("{} {} ", combo_string, combo);
    }
    info!(
        "Mutate_coefficients() finished combination {}",
        combo_string
    );
}

pub fn mutate_polynomial_from_beginning(
    base_polynomial: Polynomial,
    matrices_to_fuzz: usize,
    mutated_polynomials_to_evaluate: usize,
    generations: usize,
) -> Vec<Polynomial> {
    let current_state = initialize_current_state(&base_polynomial, mutated_polynomials_to_evaluate);
    mutate_polynomial(current_state, matrices_to_fuzz, generations)
}

pub fn mutate_polynomial(
    mut current_state: CurrentState,
    matrices_to_fuzz: usize,
    generations: usize,
) -> Vec<Polynomial> {
    info!("Starting to generate matrices to fuzz");
    let polynomial_verifier = Arc::new(polynomial_verifier::PolynomialVerifier::new(
        matrices_to_fuzz,
        current_state.starting_mutated_polynomials[0].get_size(),
    ));

    for gen in current_state.current_generation..generations {
        info!("Starting to mutate coefficients for generation {}", gen);
        let mut count = 0;
        for i in current_state.combinations_left.clone().iter() {
            for combination in i {
                if combination.is_empty() {
                    continue;
                }
                current_state
                    .interesting_polynomials
                    .append(&mut mutate_coefficients(
                        &current_state.starting_mutated_polynomials,
                        &combination,
                        &polynomial_verifier,
                    ));
                print_finished_combination(&combination);
                current_state.remove_combination(&combination, count);
            }
            info!(
                "Finished operation {} out of {}",
                count,
                current_state.combinations_left.len()
            );
            count += 1;
        }
        info!("Finished generation {}", gen);
        current_state.finish_generation();
    }
    current_state.interesting_polynomials
}

pub fn mutate_coefficients(
    polynomials: &Vec<Polynomial>,
    combination: &Vec<usize>,
    polynomial_verifier: &Arc<polynomial_verifier::PolynomialVerifier>,
) -> Vec<Polynomial> {
    let pool = ThreadPool::new(num_cpus::get()); // TODO this is something that we might want control over in the startup flags.
    let (sender, receiver): (Sender<Option<Polynomial>>, Receiver<Option<Polynomial>>) = channel();
    let number_of_polynomials = polynomials.len();
    let mut negative_polynomials = Vec::new();
    for polynomial in polynomials {
        minimize_polynomial_coefficients_async(
            polynomial.clone(),
            combination.clone(),
            &pool,
            sender.clone(),
            polynomial_verifier.clone(),
        );
    }
    for _ in 0..number_of_polynomials {
        if let Ok(Some(message)) = receiver.recv() {
            negative_polynomials.push(message);
        }
    }
    Polynomial::collapse_polynomials(&negative_polynomials)
}

pub fn minimize_polynomial_coefficients_async(
    polynomial: Polynomial,
    combination: Vec<usize>,
    pool: &ThreadPool,
    sender: Sender<Option<Polynomial>>,
    polynomial_verifier: Arc<polynomial_verifier::PolynomialVerifier>,
) {
    pool.execute(move || {
        if let Err(e) = sender.send(minimize_polynomial_coefficients(
            polynomial,
            &combination,
            &polynomial_verifier,
        )) {
            warn!("Error trying to send minimize {}", e);
        }
    });
}

pub fn minimize_polynomial_coefficients(
    mut polynomial: Polynomial,
    combination: &Vec<usize>,
    polynomial_verifier: &Arc<polynomial_verifier::PolynomialVerifier>,
) -> Option<Polynomial> {
    let mut backoff = 0.5;
    let mut did_pass = false;
    let mut old_polynomial = None;
    while backoff > 0.001 {
        if polynomial_verifier.test_polynomial(&polynomial) {
            old_polynomial = Some(polynomial.clone());
            for i in combination {
                polynomial[i.clone()] -= backoff;
            }
            did_pass = true;
        } else {
            if did_pass {
                backoff /= 2.0;
            }
            for i in combination {
                polynomial[i.clone()] += backoff;
            }
            if !did_pass {
                backoff /= 2.0;
            }
            did_pass = false;
        }
    }
    debug!(
        "Finished minimizing coefficients for {}",
        polynomial.to_string()
    );
    if let Some(polynomial) = old_polynomial {
        if !polynomial.is_polynomial_nonnegative_with_threshold(-0.1) {
            return Some(polynomial);
        }
    }
    None
}
