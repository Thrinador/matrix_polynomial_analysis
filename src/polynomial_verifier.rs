use crate::polynomial::Polynomial;
use itertools::Itertools;
use log::{debug, info, trace};
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::distributions::Uniform;
use rand::thread_rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;
use threadpool::ThreadPool;

#[derive(Debug, Clone)]
pub struct PolynomialVerifier {
    matrices: Vec<Vec<DMatrix<f64>>>,
}

impl PolynomialVerifier {
    pub fn new(number_of_matrices_to_verify: usize, matrix_size: usize) -> Self {
        let mut matrices = Vec::new();

        let start = Instant::now();
        matrices.append(&mut PolynomialVerifier::generate_circulant_matrices(
            number_of_matrices_to_verify,
            matrix_size,
        ));

        matrices.append(&mut PolynomialVerifier::generate_random_matrices(
            number_of_matrices_to_verify,
            matrix_size,
        ));

        let n_chunks = num_cpus::get();
        let mut matrix_chunks = Vec::new();
        for _ in 0..n_chunks {
            matrix_chunks.push(Vec::new());
        }
        for i in 0..matrices.len() {
            matrix_chunks[i % n_chunks].push(matrices[i].clone());
        }

        let duration = start.elapsed();
        info!("Generated matrices in {:?}", duration);
        for i in 0..matrix_chunks.len() {
            debug!("Size of chunk {}: {}", i, matrix_chunks[i].len());
        }

        PolynomialVerifier {
            matrices: matrix_chunks,
        }
    }

    pub fn test_polynomial(&self, polynomial: &Polynomial) -> bool {
        if polynomial.is_polynomial_nonnegative() {
            return true;
        }
        if polynomial.are_first_last_negative() {
            return false;
        }
        if !check_simple_matrices(polynomial) {
            return false;
        }

        let n_workers = num_cpus::get();
        let pool = ThreadPool::new(n_workers);
        let (sender, receiver): (Sender<bool>, Receiver<bool>) = channel();
        let is_alive: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

        for i in 0..n_workers {
            PolynomialVerifier::test_chunk(
                polynomial.clone(),
                &pool,
                sender.clone(),
                is_alive.clone(),
                self.matrices[i].clone(),
            );
        }

        for _ in 0..n_workers {
            if let Ok(message) = receiver.recv() {
                if !message {
                    is_alive.store(false, Ordering::Relaxed);
                    return false;
                }
            }
        }
        true
    }

    fn test_chunk(
        polynomial: Polynomial,
        pool: &ThreadPool,
        sender: Sender<bool>,
        is_alive: Arc<AtomicBool>,
        matrices: Vec<DMatrix<f64>>,
    ) {
        pool.execute(move || {
            let square_matrix_size = usize::pow(polynomial.get_size(), 2);
            for i in 0..square_matrix_size {
                for entries_to_zero in (0..square_matrix_size).combinations(i) {
                    for k in 1..matrices.len() {
                        if !is_alive.load(Ordering::Relaxed) {
                            break;
                        }
                        let mut matrix = matrices[k].clone();
                        for entry in &entries_to_zero {
                            let row = entry % polynomial.get_size();
                            let column = entry / polynomial.get_size();
                            matrix[(row, column)] = 0.0;
                        }

                        if !&polynomial.is_polynomial_nonnegative_from_matrix(&matrix) {
                            trace!("{}", &matrices[k]);
                            if let Err(e) = sender.send(false) {
                                trace!("Error trying to send distribution {}", e);
                            }
                            return;
                        }
                    }
                }
            }
            if let Err(e) = sender.send(true) {
                trace!("Error trying to send distribution {}", e);
            }
        });
    }

    fn generate_random_matrices(
        number_of_matrices_to_generate: usize,
        matrix_size: usize,
    ) -> Vec<DMatrix<f64>> {
        let mut vec = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..number_of_matrices_to_generate {
            vec.push(DMatrix::<f64>::from_distribution(
                matrix_size,
                matrix_size,
                &Uniform::<f64>::new(0.0, 100000.0),
                &mut rng,
            ));
        }
        vec
    }

    fn generate_circulant_matrices(
        number_of_matrices_to_generate: usize,
        matrix_size: usize,
    ) -> Vec<DMatrix<f64>> {
        let mut fundamental_circulant = DMatrix::<f64>::identity(matrix_size, matrix_size);
        for i in 1..matrix_size {
            fundamental_circulant.swap_rows(0, i);
        }
        let mut vec = Vec::new();
        for _ in 0..number_of_matrices_to_generate {
            let mut random_circulant = DMatrix::<f64>::zeros(matrix_size, matrix_size);
            let random_vector = DVector::<f64>::from_distribution(
                fundamental_circulant.len(),
                &Uniform::<f64>::new(1.0, 100.0),
                &mut thread_rng(),
            );
            for i in 0..fundamental_circulant.len() {
                random_circulant += random_vector[i] * random_circulant.pow(i as u32);
            }
            vec.push(random_circulant);
        }

        vec
    }
}

fn check_simple_matrices(polynomial: &Polynomial) -> bool {
    let mut identity = DMatrix::<f64>::identity(polynomial.get_size(), polynomial.get_size());
    if !&polynomial.is_polynomial_nonnegative_from_matrix(&identity) {
        return false;
    }
    // Go through some permutation matrices
    for i in 1..polynomial.get_size() {
        identity.swap_rows(0, i);
        if !&polynomial.is_polynomial_nonnegative_from_matrix(&identity) {
            return false;
        }
    }
    // Fundamental circulant
    if !&polynomial.is_polynomial_nonnegative_from_matrix(&identity) {
        return false;
    }

    true
}

pub fn is_matrix_nonnegative(matrix: &DMatrix<f64>) -> bool {
    for value in matrix.iter().enumerate() {
        if value.1 < &0.0 {
            return false;
        }
    }
    true
}
