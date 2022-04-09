use nalgebra::DMatrix;
use nalgebra::DVector;

pub fn is_matrix_nonnegative(matrix: &DMatrix<f64>) -> bool {
    for value in matrix.iter().enumerate() {
        if value.1 < &0.0 {
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
        final_matrix = final_matrix + matrix.pow(term).scale(*coefficient);
    }
    final_matrix
}

// TODO in the future it would be good to have several different distributions that these random matrices are generated from.
// The ones that come to mind are a distribution that favors the extremes much more then the middle and one that favors the
// middle (maybe gaussian) more than the extremes
pub fn fuzz_polynomial(polynomial: &DVector<f64>, size: usize) -> Option<DMatrix<f64>> {
    for _ in 1..100000 {
        let random_matrix = DMatrix::<f64>::new_random(size, size);
        let final_matrix = apply_polynomial(&polynomial, &random_matrix);
        if !is_matrix_nonnegative(&final_matrix) {
            return Some(random_matrix);
        }
    }

    None
}
