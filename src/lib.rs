use crate::polynomial::Polynomial;
use itertools::Itertools;
use nalgebra::DMatrix;
use rand::prelude::Rng;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use threadpool::ThreadPool;

mod fuzz_polynomial;
pub mod polynomial;

const RANDOM_POLYNOMIAL_MUTATIONS: usize = 5;

pub fn is_matrix_nonnegative(matrix: &DMatrix<f64>) -> bool {
    for value in matrix.iter().enumerate() {
        if value.1 < &0.0 {
            return false;
        }
    }
    true
}

fn generate_mutated_polynomials(
    polynomial_length: usize,
    matrix_size: usize,
    combination: &Vec<usize>,
) -> Vec<Polynomial> {
    let mut mutated_base_polynomials = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..RANDOM_POLYNOMIAL_MUTATIONS {
        let mut polynomial_1 = Polynomial::from_element(polynomial_length, matrix_size, 1.0);
        let mut polynomial_0 = Polynomial::from_element(polynomial_length, matrix_size, 0.0);
        for j in combination.clone() {
            let random_number_0: f64 = rng.gen();
            let random_number_1: f64 = rng.gen();
            polynomial_0[j] += random_number_0;
            polynomial_1[j] -= random_number_1;
        }
        mutated_base_polynomials.push(polynomial_0);
        mutated_base_polynomials.push(polynomial_1);
    }
    mutated_base_polynomials
}

// Given a polynomial we want to mutate it by slowly changing the coefficients of the polynomial trying to keep it
// preserving the nonnegativity of matrices of a given size. I want to do this to try and "map out" what the space of polynomials that preserve nonnegativity are.
// This space should form a cone.
//
// I think the way I want to do this is with multi-sets. First mutate all the coefficients one at a time. Next take all
// the pairs of coefficients and try to minimize them. Keep going till you try and minimize them all at once.
//
// Another idea is to write some machine learning algorithm that trys to minimize on certain criteria such as smallest
// difference between largest and smallest coeffiecents, or lowest possible negative values.
pub fn mutate_polynomial(polynomial_length: usize, matrix_size: usize) -> Vec<Polynomial> {
    let mut vector: Vec<Polynomial> = Vec::new();
    for i in 1..polynomial_length {
        let combinations_of_i = (0..polynomial_length).combinations(i);
        for combination in combinations_of_i {
            let start = Instant::now();
            vector.append(&mut mutate_coefficients(
                generate_mutated_polynomials(polynomial_length, matrix_size, &combination),
                &combination,
            ));
            let duration = start.elapsed();
            print!("Time elapsed in mutate_coefficients() with combination ");
            for combo in combination {
                print!("{} ", combo);
            }
            println!(" is: {:?}", duration);
        }
        println!("Finished operation {} out of {}", i, polynomial_length);
    }
    collapse_polynomials(vector)
}

// Returns a subset of the vector containing the elementwise smallest polynomials.
// TODO This function could use some work. It is very slow and quite long for what it is doing.
pub fn collapse_polynomials(mut polynomials: Vec<Polynomial>) -> Vec<Polynomial> {
    // Scale up polynomials so that their smallest element is one.
    for i in 0..polynomials.len() {
        let smallest_value = polynomials[i].min_term();
        for j in 0..polynomials[i].len() {
            polynomials[i][j] /= smallest_value;
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

    // Scale down polynomials so that their largest element is one.
    for i in 0..polynomials.len() {
        let largest_value = polynomials[i].max_term();
        for j in 0..polynomials[i].len() {
            polynomials[i][j] /= largest_value;
        }
    }

    polynomials
}

// Given a base polynomial of all ones make random_polynomial_mutations number of mutated polynomials with reduced coefficients.
// Then try and minimize the coefficients of those mutated polynomials.
pub fn mutate_coefficients(
    base_polynomials: Vec<Polynomial>,
    combination: &Vec<usize>,
) -> Vec<Polynomial> {
    let n_workers = 8;
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver): (Sender<Option<Polynomial>>, Receiver<Option<Polynomial>>) = channel();
    let number_of_polynomials = base_polynomials.len();
    for polynomial in base_polynomials {
        minimize_polynomial_coefficients_async(
            polynomial.clone(),
            combination.clone(),
            &pool,
            sender.clone(),
        );
    }
    let mut negative_polynomials = Vec::new();
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
) {
    pool.execute(move || {
        sender.send(minimize_polynomial_coefficients(polynomial, &combination));
    });
}

// TODO There is a bug where for some reason one polynomial of all negative coefficients except for the middle ones
// gets through.
pub fn minimize_polynomial_coefficients(
    mut polynomial: Polynomial,
    combination: &Vec<usize>,
) -> Option<Polynomial> {
    let mut backoff = 0.5;
    while backoff > 0.01 {
        for i in combination {
            polynomial[i.clone()] -= backoff;
        }
        if !fuzz_polynomial::fuzz_polynomial(&polynomial) {
            for i in combination {
                polynomial[i.clone()] += backoff;
            }
            backoff /= 2.0;
        }
    }
    if !polynomial.is_polynomial_nonnegative_with_threshold(-0.1) {
        Some(polynomial)
    } else {
        None
    }
}
