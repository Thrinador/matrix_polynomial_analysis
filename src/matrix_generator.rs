use nalgebra::DMatrix;
use nalgebra::DVector;
use rand::distributions::Uniform;
use rand::thread_rng;

pub fn generate_circulant_matrices(
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

pub fn generate_random_matrices(
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