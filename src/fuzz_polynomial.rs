use crate::{apply_polynomial, is_matrix_nonnegative, is_polynomial_nonnegative};
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::distributions::Uniform;
use rand::thread_rng;
use rand_distr::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use threadpool::ThreadPool;

const RANDOM_MATRICES_TO_GENERATE: usize = 100000;

// TODO Fuzzing should have several different distributions that these random matrices are generated from.
// The ones that come to mind are a distribution that favors the extremes much more then the middle and one that favors the
// middle (maybe gaussian) more than the extremes.

// TODO Another idea for fuzzing is to take randomly generated matrices and sometimes 0 out certain elements, or use
// different structered matrices to try and prune some early cases.

// Runs much slower than the other fuzz polynomial function, but allows for the matrix that caused the fuzz to fail to be returned.
pub fn fuzz_polynomials_slow(polynomial: &DVector<f64>, size: usize) -> Option<DMatrix<f64>> {
    // We know nonnegative polynomials are always good.
    if is_polynomial_nonnegative(polynomial) {
        return None;
    }

    let mut distributions = Vec::new();
    distributions.push(Uniform::<f64>::new(0.0, 1.0));
    distributions.push(Uniform::<f64>::new(0.0, 10.0));
    distributions.push(Uniform::<f64>::new(0.0, 1000.0));
    distributions.push(Uniform::<f64>::new(10.0, 11.0));
    distributions.push(Uniform::<f64>::new(10.0, 100.0));
    distributions.push(Uniform::<f64>::new(10000.0, 100000.0));

    for distribution in distributions {
        let mut rng = thread_rng();
        for _ in 1..RANDOM_MATRICES_TO_GENERATE {
            let random_matrix =
                DMatrix::<f64>::from_distribution(size, size, &distribution, &mut rng);
            let final_matrix = apply_polynomial(&polynomial, &random_matrix);
            if !is_matrix_nonnegative(&final_matrix) {
                return Some(final_matrix);
            }
        }
    }
    None
}

// TODO add more matrices to this function that do a good job removing problem polynomials.
fn check_structured_matrices(polynomial: &DVector<f64>, size: usize) -> bool {
    let mut identity = DMatrix::<f64>::identity(size, size);
    if !is_matrix_nonnegative(&apply_polynomial(&polynomial, &identity)) {
        return false;
    }
    // Go through some permutation matrices
    for i in 1..size {
        identity.swap_rows(0, i);
        if !is_matrix_nonnegative(&apply_polynomial(&polynomial, &identity)) {
            return false;
        }
    }
    // Fundamental circulant
    if !is_matrix_nonnegative(&apply_polynomial(&polynomial, &identity)) {
        return false;
    }

    true
}

fn are_first_last_negative(polynomial: &DVector<f64>, size: usize) -> bool {
    for i in 0..size {
        if polynomial[i] < 0.0 || polynomial[(polynomial.len() - 1) - i] < 0.0 {
            return true;
        }
    }
    false
}

pub fn fuzz_polynomial(polynomial: &DVector<f64>, size: usize) -> bool {
    // We know nonnegative polynomials are always good.
    if is_polynomial_nonnegative(polynomial) {
        return true;
    }
    if are_first_last_negative(polynomial, size) {
        return false;
    }
    if !check_structured_matrices(polynomial, size) {
        return false;
    }

    let n_workers = 8;
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver): (Sender<bool>, Receiver<bool>) = channel();

    let mut distributions = Vec::new();
    distributions.push(Uniform::<f64>::new(0.0, 1.0));
    distributions.push(Uniform::<f64>::new(0.0, 10.0));
    distributions.push(Uniform::<f64>::new(0.0, 1000.0));
    distributions.push(Uniform::<f64>::new(0.0, 100000.0));

    let mut number_of_distributions = distributions.len();

    if let Ok(dist) = InverseGaussian::<f64>::new(500.0, 10.0) {
        fuzz_polynomial_distribution_worker_inverse_gaussian(
            polynomial.clone(),
            size,
            dist,
            &pool,
            sender.clone(),
        );
        number_of_distributions += 1;
    } else {
        panic!("Error in building inverse gaussian distribution 1");
    }

    if let Ok(dist) = InverseGaussian::<f64>::new(1.0, 1.0) {
        fuzz_polynomial_distribution_worker_inverse_gaussian(
            polynomial.clone(),
            size,
            dist,
            &pool,
            sender.clone(),
        );
        number_of_distributions += 1;
    } else {
        panic!("Error in building inverse gaussian distribution 2");
    }

    for distribution in distributions {
        fuzz_polynomial_distribution_worker_uniform(
            polynomial.clone(),
            size,
            distribution,
            &pool,
            sender.clone(),
        );
    }

    for _ in 0..number_of_distributions {
        if let Ok(message) = receiver.recv() {
            if !message {
                return false;
            }
        }
    }
    true
}

fn fuzz_polynomial_distribution_worker_inverse_gaussian(
    polynomial: DVector<f64>,
    size: usize,
    dist: InverseGaussian<f64>,
    pool: &ThreadPool,
    sender: Sender<bool>,
) {
    pool.execute(move || {
        let mut rng = thread_rng();
        for _ in 1..RANDOM_MATRICES_TO_GENERATE {
            let random_matrix = DMatrix::<f64>::from_distribution(size, size, &dist, &mut rng);
            let final_matrix = apply_polynomial(&polynomial, &random_matrix);
            if !is_matrix_nonnegative(&final_matrix) {
                sender.send(false);
            }
        }
        sender.send(true);
    });
}

fn fuzz_polynomial_distribution_worker_uniform(
    polynomial: DVector<f64>,
    size: usize,
    dist: Uniform<f64>,
    pool: &ThreadPool,
    sender: Sender<bool>,
) {
    pool.execute(move || {
        let mut rng = thread_rng();
        let mut found_negative_matrix = false;
        for _ in 1..RANDOM_MATRICES_TO_GENERATE {
            let random_matrix = DMatrix::<f64>::from_distribution(size, size, &dist, &mut rng);
            let final_matrix = apply_polynomial(&polynomial, &random_matrix);
            if !is_matrix_nonnegative(&final_matrix) {
                found_negative_matrix = true;
                break;
            }
        }
        if found_negative_matrix {
            sender.send(false);
        } else {
            sender.send(true);
        }
    });
}
