#[cfg(test)]
mod tests {
    use matrix_polynomial_analysis::polynomial::Polynomial;
    use matrix_polynomial_analysis::*;

    #[test]
    fn test_collapse_polynomials() {
        // All random matrices values should be between 0 and 1.
        let mut polynomials = Vec::new();
        polynomials.push(Polynomial::from_vec(vec![2.0, 0.0, -1.0, 1.0, 1.0], 2));
        polynomials.push(Polynomial::from_vec(vec![2.0, 0.0, 1.0, 1.0, 1.0], 2));
        polynomials.push(Polynomial::from_vec(vec![3.0, 0.0, -1.0, -1.0, 1.0], 2));
        polynomials.push(Polynomial::from_vec(vec![2.0, 0.0, -1.0, 1.0, 1.0], 2));
        polynomials = collapse_polynomials(polynomials);

        println!("Polynomials length {}", polynomials.len());

        assert_eq!(polynomials.len(), 2);
        assert_eq!(
            polynomials[0],
            Polynomial::from_vec(vec![2.0, 0.0, -1.0, 1.0, 1.0], 2)
        );
        assert_eq!(
            polynomials[1],
            Polynomial::from_vec(vec![3.0, 0.0, -1.0, -1.0, 1.0], 2)
        );
    }
}
