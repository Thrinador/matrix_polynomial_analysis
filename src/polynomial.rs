use nalgebra::DMatrix;
use nalgebra::DVector;
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Polynomial {
    coefficients: DVector<f64>,
    size: usize,
}

// TODO there is a bug where randomly all polynomial coefficients turn into NaN. This needs to be fixed as it
// seemingly happens at random and will cause more problems in larger simulations. My only guess as to why it is happening
// is something to do with the sorting/collapsing of many polynomials.

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
            coefficients: DVector::from_element(polynomial_length, element),
            size: matrix_size,
        }
    }

    pub fn from_vec(vector: Vec<f64>, matrix_size: usize) -> Polynomial {
        Polynomial {
            coefficients: DVector::from_vec(vector),
            size: matrix_size,
        }
    }

    pub fn len(&self) -> usize {
        self.coefficients.len()
    }

    pub fn min_term(&self) -> f64 {
        self.coefficients.amin()
    }

    pub fn max_term(&self) -> f64 {
        self.coefficients.amax()
    }

    // TODO This function needs some work. The casting is all over the place.
    // TODO add in that if one of the last coefficients is zero, then we need to move down by matrix size.
    pub fn are_first_last_negative(&self) -> bool {
        for i in 0..self.size {
            let mut last_term: i32 = ((self.len() - 1) - i) as i32;
            if self[i] < 0.0 || self[last_term as usize] < 0.0 {
                return true;
            }
            if approx_equal(self[last_term as usize], 0.0) {
                while last_term >= 0 && approx_equal(self[last_term as usize], 0.0) {
                    if self[last_term as usize] < 0.0 {
                        return true;
                    } else if self[last_term as usize] > 0.00001 {
                        break;
                    }
                    last_term -= self.size as i32;
                }
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
