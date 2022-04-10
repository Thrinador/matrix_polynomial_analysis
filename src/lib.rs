use itertools::Itertools;
use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::prelude::*;
use rand::thread_rng;

mod fuzz_polynomial;

const random_polynomial_mutations: usize = 50;

pub fn print_polynomial(polynomial: DVector<f64>) {
    let mut i = polynomial.len();
    for term in polynomial.iter() {
        i -= 1;
        print!("+ {}x^{} ", term, i);
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
    for value in polynomial {
        // Set it slightly less than zero to deal with rounding errors
        // Make this more accurate once the fuzzing gets better.
        if value < &-0.01 {
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
            vector.append(&mut mutate_coefficients(
                &base_polynomial,
                size,
                &combination,
            ));
        }
    }
    vector
}

// One big issue with this function is that it tries to lower all the coefficients in lock step. I am not sure how to do this,
// but it would be interesting to try and lower some of them. One way this could be done is by changing the starting polynomial.
// This wouldn't give all possible variations, but might be a good place to start
pub fn mutate_coefficients(
    base_polynomial: &DVector<f64>,
    size: usize,
    combination: &Vec<usize>,
) -> Vec<DVector<f64>> {
    let mut mutated_base_polynomials = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 1..random_polynomial_mutations {
        let mut polynomial = base_polynomial.clone();
        for j in combination {
            // Generates a float between 0 and 1 and subtracts it from the base polynomial of all 1's.
            let random_number: f64 = rng.gen();
            polynomial[j.clone()] -= random_number;
        }
        mutated_base_polynomials.push(polynomial);
    }
    let mut negative_polynomials = Vec::new();
    for mut polynomial in mutated_base_polynomials {
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
        if !is_polynomial_nonnegative(&polynomial) {
            negative_polynomials.push(polynomial);
        }
    }
    negative_polynomials
}
