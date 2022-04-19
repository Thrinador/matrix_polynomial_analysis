use crate::is_matrix_nonnegative;
use crate::polynomial::Polynomial;
use itertools::Itertools;
use log::trace;
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::distributions::Uniform;
use rand::thread_rng;
use rand_distr::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use threadpool::ThreadPool;

// TODO Fuzzing should have several different distributions that these random matrices are generated from.
// The ones that come to mind are a distribution that favors the extremes much more then the middle and one that favors the
// middle (maybe gaussian) more than the extremes.

// TODO Another idea for fuzzing is to take randomly generated matrices and sometimes 0 out certain elements, or use
// different structered matrices to try and prune some early cases.
pub fn verify_polynomial(polynomial: &Polynomial, number_of_matrices_to_fuzz: usize) -> bool {
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

    let mut number_of_messages = 0;

    let square_matrix_size = usize::pow(polynomial.get_size(), 2);
    for i in 0..square_matrix_size {
        for j in (0..square_matrix_size).combinations(i) {
            number_of_messages +=
                fuzz_polynomial(polynomial, &pool, &sender, number_of_matrices_to_fuzz, j);
        }
    }

    number_of_messages += fuzz_derivatives(
        polynomial,
        &pool,
        sender.clone(),
        number_of_matrices_to_fuzz,
    );
    number_of_messages += fuzz_circulant_matrices(
        polynomial.clone(),
        &pool,
        sender.clone(),
        number_of_matrices_to_fuzz,
    );
    number_of_messages +=
        fuzz_remainder_polynomials(polynomial, &pool, &sender, number_of_matrices_to_fuzz);

    for _ in 0..number_of_messages {
        if let Ok(message) = receiver.recv() {
            if !message {
                return false;
            }
        }
    }
    true
}

fn fuzz_derivatives(
    polynomial: &Polynomial,
    pool: &ThreadPool,
    sender: Sender<bool>,
    number_of_matrices_to_fuzz: usize,
) -> usize {
    let matrix_size = polynomial.get_size();
    let mut derivative_polynomial = polynomial.clone();
    derivative_polynomial.set_size(1);
    pool.execute(move || {
        for _ in 1..matrix_size {
            derivative_polynomial = derivative_polynomial.derivative();
            if !simple_1_by_1_fuzz(&derivative_polynomial, number_of_matrices_to_fuzz) {
                sender.send(false);
                break;
            }
        }
        sender.send(true);
    });
    1
}

fn fuzz_polynomial(
    polynomial: &Polynomial,
    pool: &ThreadPool,
    sender: &Sender<bool>,
    number_of_matrices_to_fuzz: usize,
    entries_to_zero: Vec<usize>,
) -> usize {
    let mut distributions = Vec::new();
    // distributions.push(Uniform::<f64>::new(0.0, 1.0));
    distributions.push(Uniform::<f64>::new(0.0, 10.0));
    distributions.push(Uniform::<f64>::new(0.0, 1000.0));
    distributions.push(Uniform::<f64>::new(0.0, 100000.0));

    let mut number_of_distributions = distributions.len();

    if let Ok(dist) = InverseGaussian::<f64>::new(10.0, 10.0) {
        fuzz_polynomial_distribution_worker(
            polynomial.clone(),
            dist,
            &pool,
            sender.clone(),
            number_of_matrices_to_fuzz,
            entries_to_zero.clone(),
        );
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
            number_of_matrices_to_fuzz,
            entries_to_zero.clone(),
        );
    }

    number_of_distributions
}

fn generate_circulant_matrix(fundamental_circulant: &DMatrix<f64>, size: usize) -> DMatrix<f64> {
    let mut random_circulant = DMatrix::<f64>::zeros(size, size);
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
    number_of_matrices_to_fuzz: usize,
) -> usize {
    pool.execute(move || {
        let mut fundamental_circulant =
            DMatrix::<f64>::identity(polynomial.get_size(), polynomial.get_size());
        for i in 1..polynomial.get_size() {
            fundamental_circulant.swap_rows(0, i);
        }
        let mut did_pass = true;
        for _ in 0..number_of_matrices_to_fuzz {
            let random_matrix =
                generate_circulant_matrix(&fundamental_circulant, polynomial.get_size());
            if !is_matrix_nonnegative(&polynomial.apply_polynomial(&random_matrix)) {
                trace!("{}", random_matrix);
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

fn fuzz_remainder_polynomials(
    polynomial: &Polynomial,
    pool: &ThreadPool,
    sender: &Sender<bool>,
    number_of_matrices_to_fuzz: usize,
) -> usize {
    let remainder_polynomials = generate_remainder_polynomials(polynomial);
    let num_polynomials = remainder_polynomials.len();
    for polynomial in remainder_polynomials {
        let sender_clone = sender.clone();
        pool.execute(move || {
            sender_clone.send(simple_1_by_1_fuzz(&polynomial, number_of_matrices_to_fuzz));
        });
    }

    num_polynomials
}

fn generate_remainder_polynomials(polynomial: &Polynomial) -> Vec<Polynomial> {
    let mut remainder_polynomials = Vec::new();
    for _ in 0..polynomial.get_size() {
        let mut remainder_polynomial = polynomial.clone();
        remainder_polynomial.set_size(1);
        remainder_polynomials.push(remainder_polynomial);
    }
    for i in 0..polynomial.len() {
        for j in 0..polynomial.get_size() {
            if i + j < polynomial.len() {
                remainder_polynomials[j][i + j] = 0.0;
            }
        }
    }
    remainder_polynomials
}

fn fuzz_polynomial_distribution_worker<F>(
    polynomial: Polynomial,
    dist: F,
    pool: &ThreadPool,
    sender: Sender<bool>,
    number_of_matrices_to_fuzz: usize,
    entries_to_zero: Vec<usize>,
) where
    F: rand_distr::Distribution<f64> + 'static,
    F: Send,
{
    pool.execute(move || {
        let mut rng = thread_rng();
        let mut found_negative_matrix = false;
        for _ in 1..number_of_matrices_to_fuzz {
            let mut random_matrix = DMatrix::<f64>::from_distribution(
                polynomial.get_size(),
                polynomial.get_size(),
                &dist,
                &mut rng,
            );
            for entry in &entries_to_zero {
                let row = entry % polynomial.get_size();
                let column = entry / polynomial.get_size();
                random_matrix[(row, column)] = 0.0;
            }
            let final_matrix = polynomial.apply_polynomial(&random_matrix);
            if !is_matrix_nonnegative(&final_matrix) {
                trace!("{}", random_matrix);
                found_negative_matrix = true;
                break;
            }
        }
        sender.send(!found_negative_matrix);
    });
}

fn simple_1_by_1_fuzz(polynomial: &Polynomial, number_of_matrices_to_fuzz: usize) -> bool {
    let mut rng = thread_rng();
    for _ in 1..number_of_matrices_to_fuzz {
        let random_matrix =
            DMatrix::<f64>::from_distribution(1, 1, &Uniform::<f64>::new(0.0, 10000.0), &mut rng);
        let final_matrix = polynomial.apply_polynomial(&random_matrix);
        if !is_matrix_nonnegative(&final_matrix) {
            trace!("{}", random_matrix);
            return false;
        }
    }
    true
}
