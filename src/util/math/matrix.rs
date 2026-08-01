use core::{fmt, ops::{Add, Mul}, todo};

use crate::util::math::error::MatrixError;

pub trait Matrix<T, const R: usize, const C: usize> {
    fn new() -> Self;
    fn from(array: [[T; R]; C]) -> Self;
    fn new_diagonal(diagonal: [T; C]) -> Self;

    fn get_row_col(&self) -> (usize, usize) {
        (R, C)
    }

    fn multiply() -> Self;
}

pub struct Matrix3x3 {
    matrix: [[f32; 3]; 3],
}

impl Matrix<f32, 3, 3> for Matrix3x3 {
    fn new() -> Self {
        Self { 
            matrix: [[0.0; 3]; 3]
        }
    }

    fn from(array: [[f32; 3]; 3]) -> Self {
        Self { 
            matrix: array
        }
    }

    fn new_diagonal(diagonal: [f32; 3]) -> Self {

        Self { 
            matrix:  [
                [diagonal[0], 0.0,                 0.0],
                [0.0,         diagonal[1],         0.0],
                [0.0,         0.0,         diagonal[2]]
            ]
        }
    }

    fn multiply() -> Self {
        todo!("implement matrix multiplication")
    }
}

