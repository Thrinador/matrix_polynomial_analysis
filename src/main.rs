use log::{error, info};
use matrix_polynomial_analysis::polynomial::Polynomial;
use matrix_polynomial_analysis::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::time::Instant;

pub mod fuzz_polynomial;
pub mod polynomial;

#[derive(Serialize, Deserialize)]
struct Args {
    // Size of matrices to evaluate against
    matrix_size: usize,

    // The number of random matrices to test against for each type of test.
    matrices_to_fuzz: usize,

    // The number of mutated polynomials to generate based off of the `starting_polynomial`.
    // Note that this is not the number of polynomials returned, but instead the number to
    // start working off of. The number returned varies wildly since only polynomials with
    // negative coefficients are returned.
    mutated_polynomials_to_evaluate: usize,

    // The size of the polynomials that are being generated to map out a space. Only used in `mode` 3
    polynomial_length: usize,

    // Mode 1: Tests the `starting_polynomial` against matrices of size `matrix_size`.
    // Mode 2: Returns a set of mutated polynomials constructed from the `starting_polynomial`
    // that are likely nonnegative for matrices of size `matrix_size`.
    // Mode 3: Returns a snapshot of what the space of polynomials with `number_of_coefficients_in_polynomial`
    // terms looks like against `matrix_size` matrices returns a snapshot of what that space
    mode: usize,

    // A polynomial as a list of its coefficients. The first term is the largest i.e. [1,2,3] -> 1x^2 + 2x + 3.
    starting_polynomial: Vec<f64>,
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
    for poly in polynomials {
        println!("{}", poly.to_string());
    }
}

fn main() {
    let args = read_user_from_file();
    env_logger::init();
    let start = Instant::now();

    match args.mode {
        1 => {
            let polynomial = if args.starting_polynomial.len() == 0 {
                polynomial::Polynomial::from_vec(vec![1.0, 1.0, 1.0, 1.0, 1.0], args.matrix_size)
            } else {
                polynomial::Polynomial::from_vec(args.starting_polynomial, args.matrix_size)
            };
            let verify_polynomial =
                fuzz_polynomial::verify_polynomial(&polynomial, args.matrices_to_fuzz);
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
        2 => {
            let polynomial = if args.starting_polynomial.len() == 0 {
                Polynomial::from_vec(vec![1.0, 1.0, 1.0, 1.0, 1.0], args.matrix_size)
            } else {
                Polynomial::from_vec(args.starting_polynomial, args.matrix_size)
            };
            let mut interesting_polynomials = mutate_polynomial(
                polynomial,
                args.matrices_to_fuzz,
                args.mutated_polynomials_to_evaluate,
            );
            interesting_polynomials.sort();
            let duration = start.elapsed();
            info!("Total time elapsed generating polynomials {:?}", duration);
            print_polynomials(interesting_polynomials);
        }
        3 => {
            let polynomial_1 =
                Polynomial::from_element(args.polynomial_length, args.matrix_size, 1.0);
            let mut interesting_polynomials = mutate_polynomial(
                polynomial_1,
                args.matrices_to_fuzz,
                args.mutated_polynomials_to_evaluate,
            );
            interesting_polynomials.sort();
            let duration = start.elapsed();
            info!("Total time elapsed generating polynomials {:?}", duration);
            print_polynomials(interesting_polynomials);
        }
        _ => error!("mode must be set to 1,2, or 3"),
    }
}
