use log::info;
use matrix_polynomial_analysis::*;
use std::time::Instant;

mod fuzz_polynomial;
mod polynomial;

/*
pub fn evaluate_polynomial(polynomial: &Polynomial, size: usize) {
    if let Some(failed_matrix) = fuzz_polynomial::fuzz_polynomials_slow(&polynomial, size) {
        println!("The polynomial does not preserve the nonnegativity for:");
        println!("{}", failed_matrix.to_string());
    } else {
        println!(
            "The polynomial probably preserves nonnegativity for {} by {} matrices",
            size, size
        );
    }
}

pub fn test_polynomials() {
    let polynomial = DVector::from_vec(vec![2.0, 0.0, -1.0, 1.0, 1.0]);
    evaluate_polynomial(&polynomial, 2);
    evaluate_polynomial(&polynomial, 3);

    let polynomial_2 = DVector::from_vec(vec![2.0, 0.0, 0.0, -1.0, 1.0, 1.0, 1.0]);
    evaluate_polynomial(&polynomial_2, 2);
    evaluate_polynomial(&polynomial_2, 3);
}
*/

fn main() {
    env_logger::init();
    let start = Instant::now();
    let mut interesting_polynomials = mutate_polynomial(5, 2);
    interesting_polynomials.sort();
    let duration = start.elapsed();
    info!("Total time elapsed generating polynomials {:?}", duration);
    info!(
        "Total number of interesting polynomials found {}",
        interesting_polynomials.len()
    );

    // TODO should have some command line arguments to set how this gets outputted.
    for poly in interesting_polynomials {
        println!("{}", poly.to_string());
    }
}
