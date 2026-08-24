use micromath::vector::F32x2;

use crate::util::math::matrix::{Matrix, matrix2x2::Matrix2x2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix2x1 {
    matrix: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix1x2 {
    matrix: [f32; 2],
}

impl Matrix2x1 {
    pub const ZERO: Self = Self { matrix: [0.0, 0.0] };

    pub fn new() -> Self {
        Self::ZERO
    }

    pub fn from_array(array: [f32; 2]) -> Self {
        Self { matrix: array }
    }

    /// Build from a micromath vector.
    pub fn from_vector(v: F32x2) -> Self {
        Self { matrix: [v.x, v.y] }
    }

    /// Convert to a micromath vector.
    pub fn to_vector(self) -> F32x2 {
        F32x2 {
            x: self.matrix[0],
            y: self.matrix[1],
        }
    }

    pub fn as_array(&self) -> &[f32; 2] {
        &self.matrix
    }

    pub fn get_row_col(&self) -> (usize, usize) {
        (2, 1)
    }
 
    pub fn get(&self, row: usize) -> f32 {
        self.matrix[row]
    }

    pub fn scale(&self, rhs: f32) -> Self {
        Self {
            matrix: [self.matrix[0] * rhs, self.matrix[1] * rhs],
        }
    }

    pub fn transpose(&self) -> Matrix1x2 {
        Matrix1x2 {
            matrix: self.matrix,
        }
    }

    pub fn magnitude(&self) -> f32 {
        libm::sqrtf(self.matrix[0] * self.matrix[0] + self.matrix[1] * self.matrix[1])
    }
}

impl Default for Matrix2x1 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Matrix1x2 {
    pub const ZERO: Self = Self { matrix: [0.0, 0.0] };

    pub fn new() -> Self {
        Self::ZERO
    }

    pub fn from_array(array: [f32; 2]) -> Self {
        Self { matrix: array }
    }

    pub fn as_array(&self) -> &[f32; 2] {
        &self.matrix
    }

    pub fn get_row_col(&self) -> (usize, usize) {
        (1, 2)
    }

    pub fn get(&self, col: usize) -> f32 {
        self.matrix[col]
    }

    pub fn scale(&self, rhs: f32) -> Self {
        Self {
            matrix: [self.matrix[0] * rhs, self.matrix[1] * rhs],
        }
    }

    /// Transpose back into a 2x1 column matrix.
    pub fn transpose(&self) -> Matrix2x1 {
        Matrix2x1 {
            matrix: self.matrix,
        }
    }
}

impl Default for Matrix1x2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl core::ops::Add for Matrix2x1 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] + rhs.matrix[0],
                self.matrix[1] + rhs.matrix[1],
            ],
        }
    }
}

impl core::ops::Sub for Matrix2x1 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] - rhs.matrix[0],
                self.matrix[1] - rhs.matrix[1],
            ],
        }
    }
}

impl core::ops::Neg for Matrix2x1 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            matrix: [-self.matrix[0], -self.matrix[1]],
        }
    }
}

impl core::ops::Add for Matrix1x2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] + rhs.matrix[0],
                self.matrix[1] + rhs.matrix[1],
            ],
        }
    }
}

impl core::ops::Sub for Matrix1x2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] - rhs.matrix[0],
                self.matrix[1] - rhs.matrix[1],
            ],
        }
    }
}

impl core::ops::Mul<f32> for Matrix2x1 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        self.scale(rhs)
    }
}

impl core::ops::Mul<Matrix2x1> for f32 {
    type Output = Matrix2x1;
    fn mul(self, rhs: Matrix2x1) -> Matrix2x1 {
        rhs.scale(self)
    }
}

impl core::ops::Mul<f32> for Matrix1x2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        self.scale(rhs)
    }
}

impl core::ops::Mul<Matrix2x1> for Matrix2x2 {
    type Output = Matrix2x1;
    fn mul(self, rhs: Matrix2x1) -> Matrix2x1 {
        let m = self.as_array();
        Matrix2x1 {
            matrix: [
                m[0][0] * rhs.matrix[0] + m[0][1] * rhs.matrix[1],
                m[1][0] * rhs.matrix[0] + m[1][1] * rhs.matrix[1],
            ],
        }
    }
}

impl core::ops::Mul<Matrix2x2> for Matrix1x2 {
    type Output = Matrix1x2;
    fn mul(self, rhs: Matrix2x2) -> Matrix1x2 {
        let m = rhs.as_array();
        Matrix1x2 {
            matrix: [
                self.matrix[0] * m[0][0] + self.matrix[1] * m[1][0],
                self.matrix[0] * m[0][1] + self.matrix[1] * m[1][1],
            ],
        }
    }
}

impl core::ops::Mul<Matrix2x1> for Matrix1x2 {
    type Output = f32;
    fn mul(self, rhs: Matrix2x1) -> f32 {
        self.matrix[0] * rhs.matrix[0] + self.matrix[1] * rhs.matrix[1]
    }
}

impl core::ops::Mul<Matrix1x2> for Matrix2x1 {
    type Output = Matrix2x2;
    fn mul(self, rhs: Matrix1x2) -> Matrix2x2 {
        Matrix2x2::from_array([
            [self.matrix[0] * rhs.matrix[0], self.matrix[0] * rhs.matrix[1]],
            [self.matrix[1] * rhs.matrix[0], self.matrix[1] * rhs.matrix[1]],
        ])
    }
}