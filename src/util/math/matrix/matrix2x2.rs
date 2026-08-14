use micromath::vector::{F32x2};

use crate::util::math::{error::MatrixError, matrix::Matrix};


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix2x2 {
    matrix: [[f32; 2]; 2],
}

impl Matrix2x2 {
    pub const IDENTITY: Self = Self {
        matrix: [
            [1.0, 0.0],
            [0.0, 1.0],
        ],
    };

    /// Access the backing array.
    pub fn as_array(&self) -> &[[f32; 2]; 2] {
        &self.matrix
    }
}

impl Matrix<f32, 2, 2> for Matrix2x2 {
    fn new() -> Self {
        Self {
            matrix: [[0.0; 2]; 2],
        }
    }

    fn from_array(array: [[f32; 2]; 2]) -> Self {
        Self { matrix: array }
    }

    fn new_diagonal(diagonal: [f32; 2]) -> Self {
        Self {
            matrix: [
                [diagonal[0], 0.0],
                [0.0, diagonal[1]],
            ],
        }
    }

    fn transpose(&self) -> Self {
        let mut res = [[0.0f32; 2]; 2];
        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.matrix[j][i];
            }
        }
        Self { matrix: res }
    }

    fn scale(&self, rhs: f32) -> Self {
        let mut res = self.matrix;
        for row in res.iter_mut() {
            for val in row.iter_mut() {
                *val *= rhs;
            }
        }
        Self { matrix: res }
    }

    fn determinant(&self) -> f32 {
        let (a, b) = (self.matrix[0][0], self.matrix[0][1]);
        let (c, d) = (self.matrix[1][0], self.matrix[1][1]);

        a * d - b * c 
    }

    fn cofactor(&self) -> Self {
        let (a, b) = (self.matrix[0][0], self.matrix[0][1]);
        let (c, d) = (self.matrix[1][0], self.matrix[1][1]);

        Self {
            matrix: [
                [d, -c],
                [-b, a],
            ]
        }
    }

    fn adjugate(&self) -> Self {
        self.cofactor().transpose()
    }

    fn inverse(&self) -> Result<Self, MatrixError> {
        let det = self.determinant();

        if det == 0.0 || !det.is_finite() {
            return Err(MatrixError::Singular);
        }

        // A^-1 = adj(A) / det(A)
        Ok(self.adjugate().scale(1.0 / det))
    }
}

impl core::ops::Sub for Matrix2x2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = [[0.0f32; 2]; 2];
        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.matrix[i][j] - rhs.matrix[i][j];
            }
        }
        Self { matrix: res }
    }
}

impl core::ops::Mul for Matrix2x2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = [[0.0f32; 2]; 2];

        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                for k in 0..2 {
                    *val += self.matrix[i][k] * rhs.matrix[k][j];
                }
            }
        }

        Self { matrix: res }
    }
}

impl core::ops::Add for Matrix2x2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = [[0.0f32; 2]; 2];
        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.matrix[i][j] + rhs.matrix[i][j];
            }
        }
        Self { matrix: res }
    }
}

impl core::ops::Mul<f32> for Matrix2x2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Matrix::scale(&self, rhs)
    }
}

impl core::ops::Mul<F32x2> for Matrix2x2 {
    type Output = F32x2;

    fn mul(self, rhs: F32x2) -> Self::Output {
        F32x2 {
            x: self.matrix[0][0] * rhs.x + self.matrix[0][1] * rhs.y,
            y: self.matrix[1][0] * rhs.x + self.matrix[1][1] * rhs.y,
        }
    }
}
