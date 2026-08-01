use core::{fmt, ops::{Add, Mul}, todo};

use crate::util::math::error::MatrixError;

pub trait Matrix<T, const R: usize, const C: usize> {
    fn new() -> Self;
    fn from_array(array: [[T; R]; C]) -> Self;
    fn new_diagonal(diagonal: [T; C]) -> Self;

    fn get_row_col(&self) -> (usize, usize) {
        (R, C)
    }

    fn transpose(&mut self) -> Self;
    /// scalar multiplication
    fn scale(&mut self, rhs: f32);
}

#[derive(Clone, Copy)]
pub struct Matrix3x3 {
    matrix: [[f32; 3]; 3],
}

impl Matrix3x3 { 
    pub const IDENTITY: Self = 
        Self {
            matrix: [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]};
}

impl Matrix<f32, 3, 3> for Matrix3x3 {
    fn new() -> Self {
        Self { 
            matrix: [[0.0; 3]; 3]
        }
    }

    fn from_array(array: [[f32; 3]; 3]) -> Self {
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
    
    fn transpose(&mut self) -> Self {
        for i in 0..self.matrix.len() {
            for j in (i + 1)..self.matrix.len() {
                let temp = self.matrix[i][j];
                self.matrix[i][j] = self.matrix[j][i];
                self.matrix[j][i] = temp;
            }
        }

        Self {
            matrix: self.matrix
        }
    }
    
    fn scale(&mut self, rhs: f32) {
        for row in self.matrix.iter_mut() {
            for val in row.iter_mut() {
                *val *= rhs;
            }
        }
    }
}

impl core::ops::Sub for Matrix3x3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = Matrix3x3::new();
        for (i, row) in res.matrix.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.matrix[i][j] - rhs.matrix[i][j];
            }
        }

        res
    }
}

impl core::ops::Mul for Matrix3x3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = [[0.0f32; 3]; 3];

        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                for k in 0..self.matrix.len() {
                    *val += self.matrix[i][k] * rhs.matrix[k][j];
                }
            }
        }

        // [1, 2] [1, 2] i = 0, j = 0, k= 1
        // [3, 4] [3, 4]
        // 0 = 1 * 1 + 2 * 3

        Self { 
            matrix: res
        }
    }
}

impl core::ops::Add for Matrix3x3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = Matrix3x3::new();
        for (i, row) in res.matrix.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.matrix[i][j] + rhs.matrix[i][j];
            }
        }

        res
    }
}

