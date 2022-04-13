use log::info;
use matrix_polynomial_analysis::*;
use std::time::Instant;

mod fuzz_polynomial;
mod polynomial;

fn main() {
    env_logger::init();
    let start = Instant::now();
    let mut interesting_polynomials = mutate_polynomial(5, 2);
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
