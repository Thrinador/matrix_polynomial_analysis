use matrix_polynomial_analysis::*;
use nalgebra::base::*;

mod fuzz_polynomial;

pub fn evaluate_polynomial(polynomial: &DVector<f64>, size: usize) {
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

fn main() {
    let polynomial = DVector::from_vec(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
    // evaluate_polynomial(&polynomial, 2);
    let interesting_polynomials = mutate_polynomial(&polynomial, 2);
    for poly in interesting_polynomials {
        print_polynomial(poly);
        println!();
    }
}
