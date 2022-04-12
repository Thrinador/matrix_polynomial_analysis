use nalgebra::DMatrix;
use nalgebra::DVector;
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Polynomial {
    coefficients: DVector<f64>,
    size: usize,
}

impl Polynomial {
    pub fn to_string(&self) -> String {
        let mut i = self.len();
        let mut out: String = String::new();
        for term in self.coefficients.iter() {
            i -= 1;
            out = format!("{}+ {:.3}x^{} ", out, term, i);
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
        let mut term: u32 = self.coefficients.len() as u32;
        for coefficient in self.coefficients.iter() {
            term = term - 1;
            final_matrix += matrix.pow(term).scale(*coefficient);
        }
        final_matrix
    }

    pub fn from_element(polynomial_length: usize, matrix_size: usize, element: f64) -> Polynomial {
        Polynomial {
            coefficients: DVector::from_element(polynomial_length, element),
            size: matrix_size,
        }
    }

    pub fn from_vec(vector: Vec<f64>) -> Polynomial {
        let size = vector.len();
        Polynomial {
            coefficients: DVector::from_vec(vector),
            size: size,
        }
    }

    pub fn len(&self) -> usize {
        self.coefficients.len()
    }

    pub fn min_term(&self) -> f64 {
        self.coefficients[self.coefficients.iamin()]
    }

    pub fn max_term(&self) -> f64 {
        self.coefficients[self.coefficients.iamax()]
    }

    pub fn are_first_last_negative(&self) -> bool {
        for i in 0..self.size {
            if self[i] < 0.0 || self[(self.len() - 1) - i] < 0.0 {
                return true;
            }
        }
        false
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
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
