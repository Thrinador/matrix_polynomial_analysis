#[cfg(test)]
mod tests {
    use matrix_polynomial_analysis::fuzz_polynomial::*;
    use matrix_polynomial_analysis::polynomial::Polynomial;

    #[test]
    fn test_fuzz_polynomial() {
        assert_eq!(
            fuzz_polynomial(&Polynomial::from_vec(vec![3.0, 0.0, -1.0, -1.0, 1.0], 2)),
            false
        );
        assert_eq!(
            fuzz_polynomial(&Polynomial::from_vec(vec![3.0, 0.0, 1.0, 1.0, 1.0], 2)),
            true
        );
        assert_eq!(
            fuzz_polynomial(&Polynomial::from_vec(vec![-3.0, 0.0, -1.0, -1.0, 1.0], 2)),
            false
        );
        assert_eq!(
            fuzz_polynomial(&Polynomial::from_vec(vec![-1.0, -1.0, 1.0, -1.0, -1.0], 2)),
            false
        );
    }

    #[test]
    fn test_is_polynomial_nonnegative() {
        assert_eq!(
            Polynomial::from_vec(vec![3.0, 0.0, -1.0, -1.0, 1.0], 2).is_polynomial_nonnegative(),
            false
        );
        assert_eq!(
            Polynomial::from_vec(vec![3.0, 0.0, 1.0, 1.0, 1.0], 2).is_polynomial_nonnegative(),
            true
        );
        assert_eq!(
            Polynomial::from_vec(vec![-3.0, 0.0, -1.0, 1.0, 1.0], 2).is_polynomial_nonnegative(),
            false
        );
        assert_eq!(
            Polynomial::from_vec(vec![-1.0, -1.0, 1.0, -1.0, -1.0], 2).is_polynomial_nonnegative(),
            false
        );
    }
}
