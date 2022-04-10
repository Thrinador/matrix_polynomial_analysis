use crate::{apply_polynomial, is_matrix_nonnegative, is_polynomial_nonnegative};
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::distributions::Uniform;
use rand::thread_rng;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use threadpool::ThreadPool;

const random_matrices_to_generate: usize = 10000;

// TODO Fuzzing should have several different distributions that these random matrices are generated from.
// The ones that come to mind are a distribution that favors the extremes much more then the middle and one that favors the
// middle (maybe gaussian) more than the extremes

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
        for _ in 1..random_matrices_to_generate {
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

pub fn fuzz_polynomial(polynomial: &DVector<f64>, size: usize) -> bool {
    // We know nonnegative polynomials are always good.
    if is_polynomial_nonnegative(polynomial) {
        return true;
    }

    let n_workers = 8;
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver): (Sender<bool>, Receiver<bool>) = channel();

    let mut distributions = Vec::new();
    distributions.push(Uniform::<f64>::new(0.0, 1.0));
    distributions.push(Uniform::<f64>::new(0.0, 10.0));
    distributions.push(Uniform::<f64>::new(0.0, 1000.0));
    distributions.push(Uniform::<f64>::new(0.0, 100000.0));
    distributions.push(Uniform::<f64>::new(10.0, 11.0));
    distributions.push(Uniform::<f64>::new(10.0, 100.0));
    distributions.push(Uniform::<f64>::new(10000.0, 100000.0));
    distributions.push(Uniform::<f64>::new(1000000.0, 100000000.0));

    let number_of_distributions = distributions.len();
    for distribution in distributions {
        fuzz_polynomial_distribution_worker_uniform(
            polynomial.clone(),
            size,
            distribution,
            &pool,
            sender.clone(),
        );
    }

    for _ in 1..number_of_distributions {
        if let Ok(message) = receiver.recv() {
            if !message {
                return false;
            }
        }
    }
    true
}

fn fuzz_polynomial_distribution_worker_uniform(
    polynomial: DVector<f64>,
    size: usize,
    dist: rand::distributions::Uniform<f64>,
    pool: &ThreadPool,
    sender: std::sync::mpsc::Sender<bool>,
) {
    pool.execute(move || {
        let mut rng = thread_rng();
        for _ in 1..random_matrices_to_generate {
            let random_matrix = DMatrix::<f64>::from_distribution(size, size, &dist, &mut rng);
            let final_matrix = apply_polynomial(&polynomial, &random_matrix);
            if !is_matrix_nonnegative(&final_matrix) {
                sender.send(false);
            }
        }
        sender.send(true);
    });
}
