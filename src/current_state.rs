use crate::polynomial::Polynomial;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub struct CurrentState {
    pub starting_mutated_polynomials: Vec<Polynomial>,
    pub combinations_left: Vec<Vec<Vec<usize>>>,
    pub interesting_polynomials: Vec<Polynomial>,
    pub current_generation: usize,
}

impl CurrentState {
    pub fn new(polynomial_length: usize, current_generation: usize) -> Self {
        let mut combinations_left = Vec::new();
        for i in 0..polynomial_length {
            combinations_left.push(Vec::new());
            for j in (0..polynomial_length).combinations(i) {
                combinations_left[i].push(j);
            }
        }

        CurrentState {
            starting_mutated_polynomials: Vec::new(),
            combinations_left: combinations_left,
            interesting_polynomials: Vec::new(),
            current_generation,
        }
    }

    pub fn remove_combination(&mut self, combination: &Vec<usize>, combination_length: usize) {
        self.combinations_left[combination_length].retain(|x| x != combination);
        self.save_state();
    }

    pub fn finish_generation(&mut self) {
        self.interesting_polynomials =
            Polynomial::collapse_polynomials(&self.interesting_polynomials);
        self.starting_mutated_polynomials = self.interesting_polynomials.clone();
        self.current_generation += 1;
        self.save_state();
    }

    pub fn save_state(&self) {
        let json_object =
            serde_json::to_string(&self).expect("Object will be converted to JSON string");
        File::create("state.json").expect("file should open read only");
        fs::write("state.json", json_object).expect("file should open read only");
    }

    pub fn load_state() -> Self {
        let file = File::open("state.json").expect("file should open read only");
        serde_json::from_reader(file).expect("File was not able to be read")
    }
}
