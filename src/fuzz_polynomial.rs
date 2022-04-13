use crate::is_matrix_nonnegative;
use crate::polynomial::Polynomial;
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::distributions::Uniform;
use rand::thread_rng;
use rand_distr::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use threadpool::ThreadPool;

const RANDOM_MATRICES_TO_GENERATE: usize = 1000;

// TODO Fuzzing should have several different distributions that these random matrices are generated from.
// The ones that come to mind are a distribution that favors the extremes much more then the middle and one that favors the
// middle (maybe gaussian) more than the extremes.

// TODO Another idea for fuzzing is to take randomly generated matrices and sometimes 0 out certain elements, or use
// different structered matrices to try and prune some early cases.

// TODO Benchmark the performance difference of returning a bool vs returning the matrix that caused the fuzz to fail. It would be nice to combine
// the fuzz_polynomials_slow() with fuzz_polynomials() functions assume the performance hit is negligible.

// Even in its current state this fuzzing is far to slow. I think it needs a combination of failing faster and maybe some more smarts on minimizing coefficients.

// TODO This function needs to be much faster. I am not sure if that means checking fewer distributions.
// We could use more "simple" checks. Things like taking the derivative of the polynomial and fuzzing that against smaller matrices.
// Or checking sums of different parts of the coefficients like in the 2x2 case all the even coefficients must sum to be nonnegative
// same with odd
pub fn verify_polynomial(polynomial: &Polynomial) -> bool {
    // We know nonnegative polynomials are always good.
    if polynomial.is_polynomial_nonnegative() {
        return true;
    }
    if polynomial.are_first_last_negative() {
        return false;
    }
    if !check_simple_matrices(polynomial) {
        return false;
    }

    let n_workers = 8;
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver): (Sender<bool>, Receiver<bool>) = channel();

    let mut number_of_messages = fuzz_polynomial(polynomial, &pool, &sender);
    number_of_messages += fuzz_derivatives(polynomial, &pool, sender.clone());
    number_of_messages += fuzz_circulant_matrices(polynomial.clone(), &pool, sender.clone());

    for _ in 0..number_of_messages {
        if let Ok(message) = receiver.recv() {
            if !message {
                return false;
            }
        }
    }
    true
}

fn fuzz_derivatives(polynomial: &Polynomial, pool: &ThreadPool, sender: Sender<bool>) -> usize {
    let matrix_size = polynomial.get_size();
    let mut derivative_polynomial = polynomial.clone();
    derivative_polynomial.set_size(1);
    pool.execute(move || {
        for i in 1..matrix_size {
            derivative_polynomial = derivative_polynomial.derivative();
            let mut rng = thread_rng();
            let mut did_pass = true;
            for _ in 1..RANDOM_MATRICES_TO_GENERATE {
                let random_matrix = DMatrix::<f64>::from_distribution(
                    1,
                    1,
                    &Uniform::<f64>::new(0.0, 10.0),
                    &mut rng,
                );
                let final_matrix = derivative_polynomial.apply_polynomial(&random_matrix);
                if !is_matrix_nonnegative(&final_matrix) {
                    did_pass = false;
                }
            }
            if !did_pass {
                sender.send(false);
                break;
            }
        }
        sender.send(true);
    });
    1
}

fn fuzz_polynomial(polynomial: &Polynomial, pool: &ThreadPool, sender: &Sender<bool>) -> usize {
    let mut distributions = Vec::new();
    distributions.push(Uniform::<f64>::new(0.0, 1.0));
    distributions.push(Uniform::<f64>::new(0.0, 10.0));
    distributions.push(Uniform::<f64>::new(0.0, 1000.0));
    distributions.push(Uniform::<f64>::new(0.0, 100000.0));

    let mut number_of_distributions = distributions.len();

    if let Ok(dist) = InverseGaussian::<f64>::new(1.0, 1.0) {
        fuzz_polynomial_distribution_worker(polynomial.clone(), dist, &pool, sender.clone());
        number_of_distributions += 1;
    } else {
        panic!("Error in building inverse gaussian distribution 2");
    }

    for distribution in distributions {
        fuzz_polynomial_distribution_worker(
            polynomial.clone(),
            distribution,
            &pool,
            sender.clone(),
        );
    }

    number_of_distributions
}

fn generate_circulant_matrix(fundamental_circulant: &DMatrix<f64>) -> DMatrix<f64> {
    let mut random_circulant = DMatrix::<f64>::zeros(3, 3);
    let random_vector = DVector::<f64>::from_distribution(
        fundamental_circulant.len(),
        &Uniform::<f64>::new(1.0, 100.0),
        &mut thread_rng(),
    );
    for i in 0..fundamental_circulant.len() {
        random_circulant += random_vector[i] * random_circulant.pow(i as u32);
    }

    random_circulant
}

fn fuzz_circulant_matrices(
    polynomial: Polynomial,
    pool: &ThreadPool,
    sender: Sender<bool>,
) -> usize {
    pool.execute(move || {
        let mut fundamental_circulant =
            DMatrix::<f64>::identity(polynomial.get_size(), polynomial.get_size());
        for i in 1..polynomial.get_size() {
            fundamental_circulant.swap_rows(0, i);
        }
        let mut did_pass = true;
        for _ in 0..RANDOM_MATRICES_TO_GENERATE {
            if !is_matrix_nonnegative(
                &polynomial.apply_polynomial(&generate_circulant_matrix(&fundamental_circulant)),
            ) {
                did_pass = false;
                break;
            }
        }
        sender.send(did_pass);
    });
    1
}

// TODO add more matrices to this function that do a good job removing problem polynomials.
fn check_simple_matrices(polynomial: &Polynomial) -> bool {
    let mut identity = DMatrix::<f64>::identity(polynomial.get_size(), polynomial.get_size());
    if !is_matrix_nonnegative(&polynomial.apply_polynomial(&identity)) {
        return false;
    }
    // Go through some permutation matrices
    for i in 1..polynomial.get_size() {
        identity.swap_rows(0, i);
        if !is_matrix_nonnegative(&polynomial.apply_polynomial(&identity)) {
            return false;
        }
    }
    // Fundamental circulant
    if !is_matrix_nonnegative(&polynomial.apply_polynomial(&identity)) {
        return false;
    }

    true
}

fn check_sums_of_coefficients(polynomial: &Polynomial) -> bool {
    true
}

fn fuzz_polynomial_distribution_worker<F>(
    polynomial: Polynomial,
    dist: F,
    pool: &ThreadPool,
    sender: Sender<bool>,
) where
    F: rand_distr::Distribution<f64> + 'static,
    F: Send,
{
    pool.execute(move || {
        let mut rng = thread_rng();
        let mut found_negative_matrix = false;
        for _ in 1..RANDOM_MATRICES_TO_GENERATE {
            let random_matrix = DMatrix::<f64>::from_distribution(
                polynomial.get_size(),
                polynomial.get_size(),
                &dist,
                &mut rng,
            );
            let final_matrix = polynomial.apply_polynomial(&random_matrix);
            if !is_matrix_nonnegative(&final_matrix) {
                found_negative_matrix = true;
                break;
            }
        }
        sender.send(!found_negative_matrix);
    });
}
