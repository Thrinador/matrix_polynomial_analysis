use itertools::Itertools;
use log::{debug, info, warn};
use nalgebra::DMatrix;
use polynomial::Polynomial;
use rand::prelude::Rng;
use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use threadpool::ThreadPool;

pub mod polynomial;
pub mod polynomial_verifier;

#[derive(Serialize, Deserialize)]
struct CurrentState {
    starting_mutated_polynomials: Vec<Polynomial>,
    combinations_left: Vec<Vec<usize>>,
    interesting_polynomials: Vec<Polynomial>,
}

pub fn is_matrix_nonnegative(matrix: &DMatrix<f64>) -> bool {
    for value in matrix.iter().enumerate() {
        if value.1 < &0.0 {
            return false;
        }
    }
    true
}

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

pub fn mutate_polynomial(
    base_polynomial: Polynomial,
    matrices_to_fuzz: usize,
    mutated_polynomials_to_evaluate: usize,
) -> Vec<Polynomial> {
    let mutated_polynomials =
        generate_mutated_polynomials(&base_polynomial, mutated_polynomials_to_evaluate);
    debug!("Generated mutated polynomials:");
    for poly in &mutated_polynomials {
        debug!("{}", poly.to_string());
    }

    info!("Starting to generate matrices to fuzz");
    let polynomial_verifier = Arc::new(polynomial_verifier::PolynomialVerifier::new(
        matrices_to_fuzz,
        base_polynomial.get_size(),
    ));

    info!("Starting to mutate coefficients");
    let mut vector: Vec<Polynomial> = Vec::new();
    for i in 1..base_polynomial.len() {
        let combinations_of_i = (0..base_polynomial.len()).combinations(i);
        for combination in combinations_of_i {
            vector.append(&mut mutate_coefficients(
                mutated_polynomials.clone(),
                &combination,
                &polynomial_verifier,
            ));
            let mut combo_string = String::new();
            for combo in combination {
                combo_string = format!("{} {} ", combo_string, combo);
            }
            info!(
                "Mutate_coefficients() finished combination {}",
                combo_string
            );
        }
        info!("Finished operation {} out of {}", i, base_polynomial.len());
    }
    collapse_polynomials(vector)
}

// Returns a subset of the vector containing the elementwise smallest polynomials.
pub fn collapse_polynomials(mut polynomials: Vec<Polynomial>) -> Vec<Polynomial> {
    // Scale down polynomials so that their largest element is one.
    for i in 0..polynomials.len() {
        let largest_value = polynomials[i].max_term().abs();
        for j in 0..polynomials[i].len() {
            polynomials[i][j] /= largest_value;
        }
    }

    let mut i = 0;
    while i < polynomials.len() {
        let mut j = 0;
        let mut was_removed = false;
        while j < polynomials.len() {
            if i == j {
                j += 1;
                if j == polynomials.len() {
                    break;
                };
            }
            let mut bool_is_smaller_polynomial = true;
            for k in 0..polynomials[i].len() {
                if polynomials[i][k] > polynomials[j][k] {
                    bool_is_smaller_polynomial = false;
                    break;
                }
            }
            if bool_is_smaller_polynomial {
                polynomials.remove(j);
                was_removed = true;
                break;
            } else {
                j += 1;
            }
        }
        if !was_removed {
            i += 1;
        }
    }
    polynomials
}

// Given a base polynomial of all ones make random_polynomial_mutations number of mutated polynomials with reduced coefficients.
// Then try and minimize the coefficients of those mutated polynomials.
pub fn mutate_coefficients(
    base_polynomials: Vec<Polynomial>,
    combination: &Vec<usize>,
    polynomial_verifier: &Arc<polynomial_verifier::PolynomialVerifier>,
) -> Vec<Polynomial> {
    let n_workers = num_cpus::get();
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver): (Sender<Option<Polynomial>>, Receiver<Option<Polynomial>>) = channel();
    let number_of_polynomials = base_polynomials.len();
    let mut negative_polynomials = Vec::new();
    for polynomial in base_polynomials {
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
    collapse_polynomials(negative_polynomials)
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
            for i in combination {
                polynomial[i.clone()] -= backoff;
            }
            did_pass = true;
            old_polynomial = Some(polynomial.clone());
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
    if let Some(polynomial) = old_polynomial {
        if !polynomial.is_polynomial_nonnegative_with_threshold(-0.1) {
            return Some(polynomial);
        }
    }
    None
}
