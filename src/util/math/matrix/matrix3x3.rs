use micromath::vector::F32x3;

use crate::util::math::{error::MatrixError, matrix::Matrix};


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3x3 {
    matrix: [[f32; 3]; 3],
}

impl Matrix3x3 {
    pub const IDENTITY: Self = Self {
        matrix: [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
    };

    pub fn as_array(&self) -> &[[f32; 3]; 3] {
        &self.matrix
    }
}

impl Matrix<f32, 3, 3> for Matrix3x3 {
    fn new() -> Self {
        Self {
            matrix: [[0.0; 3]; 3],
        }
    }

    fn from_array(array: [[f32; 3]; 3]) -> Self {
        Self { matrix: array }
    }

    fn new_diagonal(diagonal: [f32; 3]) -> Self {
        Self {
            matrix: [
                [diagonal[0], 0.0, 0.0],
                [0.0, diagonal[1], 0.0],
                [0.0, 0.0, diagonal[2]],
            ],
        }
    }

    fn transpose(&self) -> Self {
        let mut res = [[0.0f32; 3]; 3];
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
        let (a, b, c) = (self.matrix[0][0], self.matrix[0][1], self.matrix[0][2]);
        let (d, e, f) = (self.matrix[1][0], self.matrix[1][1], self.matrix[1][2]);
        let (g, h, i) = (self.matrix[2][0], self.matrix[2][1], self.matrix[2][2]);

        (a * (e * i - f * h)) - (b * (d * i - f * g)) + (c * (d * h - e * g))
    }

    fn cofactor(&self) -> Self {
        let (a, b, c) = (self.matrix[0][0], self.matrix[0][1], self.matrix[0][2]);
        let (d, e, f) = (self.matrix[1][0], self.matrix[1][1], self.matrix[1][2]);
        let (g, h, i) = (self.matrix[2][0], self.matrix[2][1], self.matrix[2][2]);

        Self {
            matrix: [
                [ (e * i - f * h), -(d * i - f * g),  (d * h - e * g)],
                [-(b * i - c * h),  (a * i - c * g), -(a * h - b * g)],
                [ (b * f - c * e), -(a * f - c * d),  (a * e - b * d)],
            ],
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

impl core::ops::Sub for Matrix3x3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = [[0.0f32; 3]; 3];
        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.matrix[i][j] - rhs.matrix[i][j];
            }
        }
        Self { matrix: res }
    }
}

impl core::ops::Mul for Matrix3x3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = [[0.0f32; 3]; 3];

        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                for k in 0..3 {
                    *val += self.matrix[i][k] * rhs.matrix[k][j];
                }
            }
        }

        Self { matrix: res }
    }
}

impl core::ops::Add for Matrix3x3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = [[0.0f32; 3]; 3];
        for (i, row) in res.iter_mut().enumerate() {
            for (j, val) in row.iter_mut().enumerate() {
                *val = self.matrix[i][j] + rhs.matrix[i][j];
            }
        }
        Self { matrix: res }
    }
}

impl core::ops::Mul<f32> for Matrix3x3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Matrix::scale(&self, rhs)
    }
}

impl core::ops::Mul<F32x3> for Matrix3x3 {
    type Output = F32x3;

    fn mul(self, rhs: F32x3) -> Self::Output {
        F32x3 {
            x: self.matrix[0][0] * rhs.x + self.matrix[0][1] * rhs.y + self.matrix[0][2] * rhs.z,
            y: self.matrix[1][0] * rhs.x + self.matrix[1][1] * rhs.y + self.matrix[1][2] * rhs.z,
            z: self.matrix[2][0] * rhs.x + self.matrix[2][1] * rhs.y + self.matrix[2][2] * rhs.z,
        }
    }
}
