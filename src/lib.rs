use itertools::Itertools;
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::prelude::Rng;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use threadpool::ThreadPool;

mod fuzz_polynomial;

const RANDOM_POLYNOMIAL_MUTATIONS: usize = 10;

pub fn print_polynomial(polynomial: DVector<f64>) {
    let mut i = polynomial.len();
    for term in polynomial.iter() {
        i -= 1;
        print!("+ {:.3}x^{} ", term, i);
    }
}

pub fn is_matrix_nonnegative(matrix: &DMatrix<f64>) -> bool {
    for value in matrix.iter().enumerate() {
        if value.1 < &0.0 {
            return false;
        }
    }
    true
}

pub fn is_polynomial_nonnegative(polynomial: &DVector<f64>) -> bool {
    is_polynomial_nonnegative_with_threshold(&polynomial, 0.0)
}

pub fn is_polynomial_nonnegative_with_threshold(polynomial: &DVector<f64>, threshold: f64) -> bool {
    for value in polynomial {
        if value < &threshold {
            return false;
        }
    }
    true
}

// As things get larger this function will need to be optimized. There might be some tricks with moving the powers of the
// matrix up as we go instead of taking a new power each time.
pub fn apply_polynomial(polynomial: &DVector<f64>, matrix: &DMatrix<f64>) -> DMatrix<f64> {
    let mut final_matrix = matrix.scale(0.0);
    let mut term: u32 = polynomial.len() as u32;
    for coefficient in polynomial.iter() {
        term = term - 1;
        final_matrix += matrix.pow(term).scale(*coefficient);
    }
    final_matrix
}

fn generate_mutated_polynomials(
    base_polynomial: &DVector<f64>,
    combination: &Vec<usize>,
) -> Vec<DVector<f64>> {
    let mut mutated_base_polynomials = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..RANDOM_POLYNOMIAL_MUTATIONS {
        for j in combination.clone() {
            let mut polynomial = base_polynomial.clone();
            // Generates a float between 0 and 1 and subtracts it from the base polynomial of all 1's.
            let random_number: f64 = rng.gen();
            polynomial[j] -= random_number;
            mutated_base_polynomials.push(polynomial);
        }
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
pub fn mutate_polynomial(base_polynomial: &DVector<f64>, size: usize) -> Vec<DVector<f64>> {
    let mut vector: Vec<DVector<f64>> = Vec::new();
    for i in 1..base_polynomial.len() {
        let combinations_of_i = (0..base_polynomial.len()).combinations(i);
        for combination in combinations_of_i {
            let start = Instant::now();
            vector.append(&mut mutate_coefficients(
                generate_mutated_polynomials(&base_polynomial, &combination),
                size,
                &combination,
            ));
            let duration = start.elapsed();
            print!("Time elapsed in mutate_coefficients() with combination ");
            for combo in combination {
                print!("{} ", combo);
            }
            println!(" is: {:?}", duration);
        }
        println!("Finished operation {} out of {}", i, base_polynomial.len());
    }
    collapse_polynomials(vector)
}

// Returns a subset of the vector containing the elementwise smallest polynomials.
// TODO This function could use some work. It is very slow and quite long for what it is doing.
pub fn collapse_polynomials(mut polynomials: Vec<DVector<f64>>) -> Vec<DVector<f64>> {
    // Scale up polynomials so that their smallest element is one.
    for i in 0..polynomials.len() {
        let smallest_value = polynomials[i][polynomials[i].iamin()];
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
        let largest_value = polynomials[i][polynomials[i].iamax()];
        for j in 0..polynomials[i].len() {
            polynomials[i][j] /= largest_value;
        }
    }

    polynomials
}

// Given a base polynomial of all ones make random_polynomial_mutations number of mutated polynomials with reduced coefficients.
// Then try and minimize the coefficients of those mutated polynomials.
pub fn mutate_coefficients(
    base_polynomials: Vec<DVector<f64>>,
    size: usize,
    combination: &Vec<usize>,
) -> Vec<DVector<f64>> {
    let n_workers = 8;
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver): (Sender<Option<DVector<f64>>>, Receiver<Option<DVector<f64>>>) =
        channel();
    let number_of_polynomials = base_polynomials.len();
    for polynomial in base_polynomials {
        minimize_polynomial_coefficients_async(
            polynomial.clone(),
            size.clone(),
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
    polynomial: DVector<f64>,
    size: usize,
    combination: Vec<usize>,
    pool: &ThreadPool,
    sender: Sender<Option<DVector<f64>>>,
) {
    pool.execute(move || {
        sender.send(minimize_polynomial_coefficients(
            polynomial,
            size,
            &combination,
        ));
    });
}

// TODO There is a bug where for some reason one polynomial of all negative coefficients except for the middle ones
// gets through.
pub fn minimize_polynomial_coefficients(
    mut polynomial: DVector<f64>,
    size: usize,
    combination: &Vec<usize>,
) -> Option<DVector<f64>> {
    let mut backoff = 0.5;
    while backoff > 0.01 {
        for i in combination {
            polynomial[i.clone()] -= backoff;
        }
        if !fuzz_polynomial::fuzz_polynomial(&polynomial, size) {
            for i in combination {
                polynomial[i.clone()] += backoff;
            }
            backoff /= 2.0;
        }
    }
    if !is_polynomial_nonnegative_with_threshold(&polynomial, -0.1) {
        Some(polynomial)
    } else {
        None
    }
}
