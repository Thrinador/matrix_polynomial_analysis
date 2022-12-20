use crate::polynomial::Polynomial;
use itertools::Itertools;
use log::{info, trace};
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::distributions::Uniform;
use rand::thread_rng;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PolynomialVerifier {
    matrices: Vec<Vec<DMatrix<f64>>>,
}

impl PolynomialVerifier {
    pub fn new(number_of_matrices_to_verify: usize, matrix_size: usize, powers: usize) -> Self {
        let mut matrices = Vec::new();

        let start = Instant::now();
        matrices.append(&mut PolynomialVerifier::generate_circulant_matrices(
            number_of_matrices_to_verify,
            matrix_size,
            powers,
        ));

        /*
        matrices.append(&mut PolynomialVerifier::generate_random_matrices(
            number_of_matrices_to_verify,
            matrix_size,
        ));
        */
        let duration = start.elapsed();
        info!("Generated matrices in {:?}", duration);
        PolynomialVerifier { matrices }
    }

    // TODO throwing in random zeros is useful this also scales poorly with size of matrix revist this function with a better plan to add zeros.
    /*
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

        let square_matrix_size = usize::pow(polynomial.get_size(), 2);
        for j in 0..square_matrix_size {
            for k in 1..self.matrices.len() {
                if !&polynomial.is_polynomial_nonnegative_from_matrix(&self.matrices[k]) {
                    trace!("{}", &self.matrices[k]);
                    return false;
                }
                for entries_to_zero in (0..square_matrix_size).combinations(j) {
                    let mut matrix = self.matrices[k].clone();
                    for entry in &entries_to_zero {
                        let row = entry % polynomial.get_size();
                        let column = entry / polynomial.get_size();
                        matrix[(row, column)] = 0.0;
                    }
                    if !&polynomial.is_polynomial_nonnegative_from_matrix(&matrix) {
                        trace!("{}", &matrix);
                        return false;
                    }
                }
            }
        }
        true
    }
    */

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

        for k in 1..self.matrices.len() {
            if !&polynomial.is_polynomial_nonnegative_from_matrix_with_powers(&self.matrices[k]) {
                trace!("{}", &self.matrices[k][0]);
                return false;
            }
        }
        true
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
        powers: usize,
    ) -> Vec<Vec<DMatrix<f64>>> {
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
            let mut matrix_powers = Vec::new();
            let mut working_circulant = DMatrix::<f64>::identity(matrix_size, matrix_size);
            for _ in 0..powers {
                matrix_powers.push(working_circulant.clone());
                working_circulant *= &random_circulant;
            }
            vec.push(matrix_powers);
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
    true
}
