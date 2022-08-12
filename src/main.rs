use log::{error, info};
use matrix_polynomial_analysis::current_state::CurrentState;
use matrix_polynomial_analysis::polynomial::Polynomial;
use matrix_polynomial_analysis::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::time::Instant;

#[derive(Serialize, Deserialize)]
struct Args {
    matrix_size: usize,
    matrices_to_fuzz: usize,
    mutated_polynomials_to_evaluate: usize,
    polynomial_length: usize,
    mode: usize,
    starting_polynomial: Vec<f64>,
    number_of_generations: usize,
}

#[derive(Serialize, Deserialize)]
struct Output {
    interesting_polynomials: Vec<Polynomial>,
}

enum PolynomialMode {
    TestPolynomial,
    MutatePolynomial,
    MapSpace,
    ReturnState,
    Error,
}

fn i32_to_polynomial_mode(val: usize) -> PolynomialMode {
    match val {
        1 => PolynomialMode::TestPolynomial,
        2 => PolynomialMode::MutatePolynomial,
        3 => PolynomialMode::MapSpace,
        4 => PolynomialMode::ReturnState,
        _ => PolynomialMode::Error,
    }
}

fn read_user_from_file() -> Args {
    let file = File::open("startup.json").expect("file should open read only");
    serde_json::from_reader(file).expect("File was not able to be read")
}

fn print_polynomials(polynomials: Vec<Polynomial>) {
    info!(
        "Total number of interesting polynomials found {}",
        polynomials.len()
    );
    for poly in &polynomials {
        println!("{}", poly.to_string());
    }

    let json_object = serde_json::to_string(&Output {
        interesting_polynomials: polynomials,
    })
    .expect("Object will be converted to JSON string");
    File::create("output.json").expect("file should open read only");
    fs::write("output.json", json_object).expect("file should open read only");
}

fn mode_test_polynomial(args: Args) {
    let start = Instant::now();
    let polynomial = if args.starting_polynomial.len() == 0 {
        polynomial::Polynomial::from_vec(vec![1.0, 1.0, 1.0, 1.0, 1.0], args.matrix_size)
    } else {
        polynomial::Polynomial::from_vec(args.starting_polynomial, args.matrix_size)
    };
    let polynomial_verifier =
        polynomial_verifier::PolynomialVerifier::new(args.matrices_to_fuzz, args.matrix_size);
    let verify_polynomial = polynomial_verifier.test_polynomial(&polynomial);
    let duration = start.elapsed();
    info!("Total time elapsed verifying polynomial {:?}", duration);
    if verify_polynomial {
        println!(
            "The polynomial {} probably preserves {}-by-{} matrices.",
            polynomial.to_string(),
            args.matrix_size,
            args.matrix_size
        );
    } else {
        println!(
            "The polynomial {} does not preserves {}-by-{} matrices.",
            polynomial.to_string(),
            args.matrix_size,
            args.matrix_size
        );
    }
}

fn mode_mutate_polynomial(args: Args) {
    let start = Instant::now();
    let polynomial = if args.starting_polynomial.len() == 0 {
        Polynomial::from_vec(vec![1.0, 1.0, 1.0, 1.0, 1.0], args.matrix_size)
    } else {
        Polynomial::from_vec(args.starting_polynomial, args.matrix_size)
    };
    let mut interesting_polynomials = mutate_polynomial_from_beginning(
        polynomial,
        args.matrices_to_fuzz,
        args.mutated_polynomials_to_evaluate,
        args.number_of_generations,
    );
    interesting_polynomials.sort();
    let duration = start.elapsed();
    info!("Total time elapsed generating polynomials {:?}", duration);
    print_polynomials(interesting_polynomials);
}

fn mode_map_space(args: Args) {
    let start = Instant::now();
    let polynomial = Polynomial::from_element(args.polynomial_length, args.matrix_size, 1.0);
    let mut interesting_polynomials = mutate_polynomial_from_beginning(
        polynomial,
        args.matrices_to_fuzz,
        args.mutated_polynomials_to_evaluate,
        args.number_of_generations,
    );
    interesting_polynomials.sort();
    let duration = start.elapsed();
    info!("Total time elapsed generating polynomials {:?}", duration);
    print_polynomials(interesting_polynomials);
}

fn mode_return_state(args: Args) {
    let start = Instant::now();
    let current_state = CurrentState::load_state();
    let mut interesting_polynomials = mutate_polynomial(
        current_state,
        args.matrices_to_fuzz,
        args.number_of_generations,
    );
    interesting_polynomials.sort();
    let duration = start.elapsed();
    info!("Total time elapsed generating polynomials {:?}", duration);
    print_polynomials(interesting_polynomials);
}

fn main() {
    let args = read_user_from_file();
    env_logger::init();
    match i32_to_polynomial_mode(args.mode) {
        PolynomialMode::TestPolynomial => mode_test_polynomial(args),
        PolynomialMode::MutatePolynomial => mode_mutate_polynomial(args),
        PolynomialMode::MapSpace => mode_map_space(args),
        PolynomialMode::ReturnState => mode_return_state(args),
        PolynomialMode::Error => error!("mode must be set to 1,2, or 3"),
    }
}
