use crate::polynomial::Polynomial;
use clap::Parser;
use log::{error, info};
use matrix_polynomial_analysis::*;
use std::time::Instant;

pub mod fuzz_polynomial;
pub mod polynomial;

// TODO there is a problem with the starting_polynomial. I don't have a good way to pass it in right now

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Size of matrices to evaluate against
    #[clap(long, default_value_t = 2)]
    matrix_size: usize,

    /// The number of random matrices to test against for each type of test.
    #[clap(long, default_value_t = 1000)]
    number_of_matrices_to_fuzz: usize,

    /// The number of mutated polynomials to generate based off of the `starting_polynomial`.
    /// Note that this is not the number of polynomials returned, but instead the number to
    /// start working off of. The number returned varies wildly since only polynomials with
    /// negative coefficients are returned.
    #[clap(long, default_value_t = 10)]
    number_of_mutated_polynomials_to_evaluate: usize,

    /// The size of the polynomials that are being generated to map out a space. Only used in `mode` 3
    #[clap(long, default_value_t = 5)]
    number_of_coefficients_in_polynomial: usize,

    /// Mode 1: Tests the `starting_polynomial` against matrices of size `matrix_size`.
    /// Mode 2: Returns a set of mutated polynomials constructed from the `starting_polynomial`
    /// that are likely nonnegative for matrices of size `matrix_size`.
    /// Mode 3: Returns a snapshot of what the space of polynomials with `number_of_coefficients_in_polynomial`
    /// terms looks like against `matrix_size` matrices returns a snapshot of what that space
    #[clap(long, default_value_t = 1)]
    mode: usize,

    /// A polynomial as a list of its coefficients. The first term is the largest i.e. [1,2,3] -> 1x^2 + 2x + 3.
    #[clap(long)]
    starting_polynomial: Vec<f64>,
}

fn main() {
    let args = Args::parse();
    env_logger::init();
    let start = Instant::now();

    match args.mode {
        1 => {
            let polynomial = if args.starting_polynomial.len() == 0 {
                Polynomial::from_vec(
                    vec![1.0, 1.0, -0.636535, 0.1093750, 0.3191201],
                    args.matrix_size,
                )
            } else {
                Polynomial::from_vec(args.starting_polynomial, args.matrix_size)
            };
            let verify_polynomial =
                fuzz_polynomial::verify_polynomial(&polynomial, args.number_of_matrices_to_fuzz);
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
            println!("This operation is not yet supported");
        }
        3 => {
            let mut interesting_polynomials = mutate_polynomial(
                args.number_of_coefficients_in_polynomial,
                args.matrix_size,
                args.number_of_matrices_to_fuzz,
                args.number_of_mutated_polynomials_to_evaluate,
            );
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
        _ => error!("mode must be set to 1,2, or 3"),
    }
}
