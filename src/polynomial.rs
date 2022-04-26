use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polynomial {
    coefficients: Vec<f64>,
    size: usize,
}

impl Polynomial {
    pub fn to_string(&self) -> String {
        let mut i = self.len();
        let mut out: String = String::new();
        for term in self.coefficients.iter() {
            i -= 1;
            if term >= &0.0 {
                out = format!("{}+ {:.7}x^{} ", out, term, i);
            } else {
                out = format!("{}- {:.7}x^{} ", out, term.abs(), i);
            }
        }
        out
    }

    pub fn is_polynomial_nonnegative(&self) -> bool {
        self.is_polynomial_nonnegative_with_threshold(0.0)
    }

    pub fn is_polynomial_nonnegative_with_threshold(&self, threshold: f64) -> bool {
        for value in self.coefficients.iter() {
            if value < &threshold {
                return false;
            }
        }
        true
    }

    // As things get larger this function will need to be optimized. There might be some tricks with moving the powers of the
    // matrix up as we go instead of taking a new power each time.
    pub fn apply_polynomial(&self, matrix: &DMatrix<f64>) -> DMatrix<f64> {
        let mut final_matrix = matrix.scale(0.0);
        let mut working_matrix = DMatrix::<f64>::identity(self.size, self.size);
        for coefficient in self.coefficients.iter().rev() {
            final_matrix += working_matrix.scale(*coefficient);
            working_matrix *= matrix;
        }
        final_matrix
    }

    pub fn from_element(polynomial_length: usize, matrix_size: usize, element: f64) -> Polynomial {
        Polynomial {
            coefficients: vec![element; polynomial_length],
            size: matrix_size,
        }
    }

    pub fn from_vec(coefficients: Vec<f64>, matrix_size: usize) -> Polynomial {
        Polynomial {
            coefficients: coefficients,
            size: matrix_size,
        }
    }

    pub fn len(&self) -> usize {
        self.coefficients.len()
    }

    pub fn min_term(&self) -> f64 {
        let mut min = self.coefficients[0].abs();
        for coefficient in &self.coefficients {
            if min > coefficient.abs() {
                min = coefficient.abs();
            }
        }
        min
    }

    pub fn max_term(&self) -> f64 {
        let mut max = self.coefficients[0].abs();
        for coefficient in &self.coefficients {
            if max < coefficient.abs() {
                max = coefficient.abs();
            }
        }
        max
    }

    pub fn are_first_last_negative(&self) -> bool {
        for i in 0..self.size {
            if self.len() - 1 < i {
                break;
            }
            let mut last_term = (self.len() - 1) - i;
            let mut term = i;
            if self[i] < 0.0 || self[last_term] < 0.0 {
                return true;
            }
            while approx_equal(self[last_term], 0.0) {
                if self[last_term] < 0.0 {
                    return true;
                } else if self[last_term] > 0.0000001 {
                    break;
                } else if last_term < self.size {
                    break;
                }
                last_term -= self.size;
            }
            while approx_equal(self[term], 0.0) {
                if self[term] < 0.0 {
                    return true;
                } else if self[term] > 0.0000001 {
                    break;
                } else if term + self.size >= self.len() {
                    break;
                }
                term += self.size;
            }
        }
        false
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    pub fn derivative(&self) -> Polynomial {
        let mut derivative = Polynomial::from_element(self.len() - 1, self.size, 0.0);
        for i in 1..self.len() {
            derivative[i - 1] = (i as f64) * self[i];
        }
        derivative
    }
}

// TODO this is a pretty rough function, for now my percision caps at 3 decimals so it is sufficient.
pub fn approx_equal(term1: f64, term2: f64) -> bool {
    (term1 - term2).abs() < 0.00001
}

impl PartialEq for Polynomial {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size {
            return false;
        }
        for i in 0..self.size {
            if self[i] != other[i] {
                return false;
            }
        }
        true
    }
}

impl Eq for Polynomial {}

impl PartialOrd for Polynomial {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.size > other.size {
            return Some(Ordering::Greater);
        } else if self.size < other.size {
            return Some(Ordering::Less);
        }
        for i in 0..self.size {
            if self[i] > other[i] {
                return Some(Ordering::Greater);
            } else if self[i] < other[i] {
                return Some(Ordering::Less);
            }
        }
        Some(Ordering::Equal)
    }
}

impl Ord for Polynomial {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.size > other.size {
            return Ordering::Greater;
        } else if self.size < other.size {
            return Ordering::Less;
        }
        for i in 0..self.size {
            if self[i] > other[i] {
                return Ordering::Greater;
            } else if self[i] < other[i] {
                return Ordering::Less;
            }
        }
        Ordering::Equal
    }
}

impl Index<usize> for Polynomial {
    type Output = f64;
    fn index<'a>(&'a self, i: usize) -> &'a f64 {
        &self.coefficients[i]
    }
}

impl IndexMut<usize> for Polynomial {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut f64 {
        &mut self.coefficients[i]
    }
}
