#[cfg(test)]
mod tests {
    use matrix_polynomial_analysis::*;
    use nalgebra::base::*;

    #[test]
    fn test_is_matrix_nonnegative() {
        // All random matrices values should be between 0 and 1.
        let mut rand = DMatrix::<f64>::new_random(100, 100);

        assert_eq!(is_matrix_nonnegative(&rand), true);

        rand[(5, 5)] = -1.0;
        assert_eq!(is_matrix_nonnegative(&rand), false);
    }

    #[test]
    fn test_apply_polynomial() {
        // All random matrices values should be between 0 and 1.
        let identity = DMatrix::<f64>::identity(3, 3);

        let m1 = DMatrix::from_iterator(
            3,
            3,
            [
                // Components listed column-by-column.
                3.0, 2.0, 1.0, 1.0, 3.0, 2.0, 2.0, 1.0, 3.0,
            ]
            .iter()
            .cloned(),
        );

        let m1_p1 = DMatrix::from_iterator(
            3,
            3,
            [
                // Components listed column-by-column.
                518.0, 528.0, 509.0, 509.0, 518.0, 528.0, 528.0, 509.0, 518.0,
            ]
            .iter()
            .cloned(),
        );

        let m1_p2 = DMatrix::from_iterator(
            3,
            3,
            [
                // Components listed column-by-column.
                849.0, 865.0, 849.0, 849.0, 849.0, 865.0, 865.0, 849.0, 849.0,
            ]
            .iter()
            .cloned(),
        );

        let polynomial_1 = DVector::from_vec(vec![1.0, 1.0, 1.0, 1.0, 1.0]);
        let polynomial_2 = DVector::from_vec(vec![2.0, 0.0, -1.0, 1.0, 1.0]);

        assert_eq!(
            apply_polynomial(&polynomial_1, &identity),
            DMatrix::<f64>::identity(3, 3).scale(5.0)
        );
        assert_eq!(
            apply_polynomial(&polynomial_2, &identity),
            DMatrix::<f64>::identity(3, 3).scale(3.0)
        );

        assert_eq!(
            apply_polynomial(&polynomial_1, &identity),
            DMatrix::<f64>::identity(3, 3).scale(5.0)
        );
        assert_eq!(apply_polynomial(&polynomial_1, &m1), m1_p1);
        assert_eq!(apply_polynomial(&polynomial_2, &m1), m1_p2);
    }
}
