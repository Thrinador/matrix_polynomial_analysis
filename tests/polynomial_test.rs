#[cfg(test)]
mod tests {
    use matrix_polynomial_analysis::polynomial::Polynomial;
    use matrix_polynomial_analysis::polynomial_verifier::PolynomialVerifier;

    #[test]
    fn test_fuzz_polynomial() {
        let polynomial_verifier = PolynomialVerifier::new(10000, 2);

        assert_eq!(
            polynomial_verifier
                .test_polynomial(&Polynomial::from_vec(vec![3.0, 0.0, -1.0, -1.0, 1.0], 2)),
            false
        );
        assert_eq!(
            polynomial_verifier
                .test_polynomial(&Polynomial::from_vec(vec![3.0, 0.0, 1.0, 1.0, 1.0], 2)),
            true
        );
        assert_eq!(
            polynomial_verifier
                .test_polynomial(&Polynomial::from_vec(vec![-3.0, 0.0, -1.0, -1.0, 1.0], 2)),
            false
        );
        assert_eq!(
            polynomial_verifier
                .test_polynomial(&Polynomial::from_vec(vec![-1.0, -1.0, 1.0, -1.0, -1.0], 2)),
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
