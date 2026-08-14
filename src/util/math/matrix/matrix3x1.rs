use micromath::vector::F32x3;

use crate::util::math::matrix::{Matrix, matrix3x3::Matrix3x3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3x1 {
    matrix: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix1x3 {
    matrix: [f32; 3],
}

impl Matrix3x1 {
    pub const ZERO: Self = Self { matrix: [0.0, 0.0, 0.0] };

    pub fn new() -> Self {
        Self::ZERO
    }

    pub fn from_array(array: [f32; 3]) -> Self {
        Self { matrix: array }
    }

    pub fn from_vector(v: F32x3) -> Self {
        Self { matrix: [v.x, v.y, v.z] }
    }

    pub fn to_vector(self) -> F32x3 {
        F32x3 {
            x: self.matrix[0],
            y: self.matrix[1],
            z: self.matrix[2],
        }
    }

    pub fn as_array(&self) -> &[f32; 3] {
        &self.matrix
    }

    pub fn get_row_col(&self) -> (usize, usize) {
        (3, 1)
    }

    pub fn get(&self, row: usize) -> f32 {
        self.matrix[row]
    }

    pub fn scale(&self, rhs: f32) -> Self {
        Self {
            matrix: [
                self.matrix[0] * rhs,
                self.matrix[1] * rhs,
                self.matrix[2] * rhs,
            ],
        }
    }

    pub fn transpose(&self) -> Matrix1x3 {
        Matrix1x3 { matrix: self.matrix }
    }

    pub fn magnitude(&self) -> f32 {
        libm::sqrtf(
            self.matrix[0] * self.matrix[0]
                + self.matrix[1] * self.matrix[1]
                + self.matrix[2] * self.matrix[2],
        )
    }
}

impl Default for Matrix3x1 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Matrix1x3 {
    pub const ZERO: Self = Self { matrix: [0.0, 0.0, 0.0] };

    pub fn new() -> Self {
        Self::ZERO
    }

    pub fn from_array(array: [f32; 3]) -> Self {
        Self { matrix: array }
    }

    pub fn as_array(&self) -> &[f32; 3] {
        &self.matrix
    }

    pub fn get_row_col(&self) -> (usize, usize) {
        (1, 3)
    }

    pub fn get(&self, col: usize) -> f32 {
        self.matrix[col]
    }

    pub fn scale(&self, rhs: f32) -> Self {
        Self {
            matrix: [
                self.matrix[0] * rhs,
                self.matrix[1] * rhs,
                self.matrix[2] * rhs,
            ],
        }
    }

    pub fn transpose(&self) -> Matrix3x1 {
        Matrix3x1 { matrix: self.matrix }
    }
}

impl Default for Matrix1x3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl core::ops::Add for Matrix3x1 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] + rhs.matrix[0],
                self.matrix[1] + rhs.matrix[1],
                self.matrix[2] + rhs.matrix[2],
            ],
        }
    }
}

impl core::ops::Sub for Matrix3x1 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] - rhs.matrix[0],
                self.matrix[1] - rhs.matrix[1],
                self.matrix[2] - rhs.matrix[2],
            ],
        }
    }
}

impl core::ops::Neg for Matrix3x1 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            matrix: [-self.matrix[0], -self.matrix[1], -self.matrix[2]],
        }
    }
}

impl core::ops::Add for Matrix1x3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] + rhs.matrix[0],
                self.matrix[1] + rhs.matrix[1],
                self.matrix[2] + rhs.matrix[2],
            ],
        }
    }
}

impl core::ops::Sub for Matrix1x3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            matrix: [
                self.matrix[0] - rhs.matrix[0],
                self.matrix[1] - rhs.matrix[1],
                self.matrix[2] - rhs.matrix[2],
            ],
        }
    }
}

impl core::ops::Mul<f32> for Matrix3x1 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        self.scale(rhs)
    }
}

impl core::ops::Mul<Matrix3x1> for f32 {
    type Output = Matrix3x1;
    fn mul(self, rhs: Matrix3x1) -> Matrix3x1 {
        rhs.scale(self)
    }
}

impl core::ops::Mul<f32> for Matrix1x3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        self.scale(rhs)
    }
}

impl core::ops::Mul<Matrix3x1> for Matrix3x3 {
    type Output = Matrix3x1;
    fn mul(self, rhs: Matrix3x1) -> Matrix3x1 {
        let m = self.as_array();
        Matrix3x1 {
            matrix: [
                m[0][0] * rhs.matrix[0] + m[0][1] * rhs.matrix[1] + m[0][2] * rhs.matrix[2],
                m[1][0] * rhs.matrix[0] + m[1][1] * rhs.matrix[1] + m[1][2] * rhs.matrix[2],
                m[2][0] * rhs.matrix[0] + m[2][1] * rhs.matrix[1] + m[2][2] * rhs.matrix[2],
            ],
        }
    }
}

impl core::ops::Mul<Matrix3x3> for Matrix1x3 {
    type Output = Matrix1x3;
    fn mul(self, rhs: Matrix3x3) -> Matrix1x3 {
        let m = rhs.as_array();
        Matrix1x3 {
            matrix: [
                self.matrix[0] * m[0][0] + self.matrix[1] * m[1][0] + self.matrix[2] * m[2][0],
                self.matrix[0] * m[0][1] + self.matrix[1] * m[1][1] + self.matrix[2] * m[2][1],
                self.matrix[0] * m[0][2] + self.matrix[1] * m[1][2] + self.matrix[2] * m[2][2],
            ],
        }
    }
}

impl core::ops::Mul<Matrix3x1> for Matrix1x3 {
    type Output = f32;
    fn mul(self, rhs: Matrix3x1) -> f32 {
        self.matrix[0] * rhs.matrix[0]
            + self.matrix[1] * rhs.matrix[1]
            + self.matrix[2] * rhs.matrix[2]
    }
}

impl core::ops::Mul<Matrix1x3> for Matrix3x1 {
    type Output = Matrix3x3;
    fn mul(self, rhs: Matrix1x3) -> Matrix3x3 {
        Matrix3x3::from_array([
            [
                self.matrix[0] * rhs.matrix[0],
                self.matrix[0] * rhs.matrix[1],
                self.matrix[0] * rhs.matrix[2],
            ],
            [
                self.matrix[1] * rhs.matrix[0],
                self.matrix[1] * rhs.matrix[1],
                self.matrix[1] * rhs.matrix[2],
            ],
            [
                self.matrix[2] * rhs.matrix[0],
                self.matrix[2] * rhs.matrix[1],
                self.matrix[2] * rhs.matrix[2],
            ],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat3x3_times_col() {
        let a = Matrix3x3::from_array([
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ]);
        let v = Matrix3x1::from_array([1.0, 2.0, 3.0]);
        let r = a * v;
        assert_eq!(r.as_array(), &[14.0, 32.0, 50.0]);
    }

    #[test]
    fn inner_and_outer_products() {
        let c = Matrix3x1::from_array([2.0, 3.0, 4.0]);
        let r = Matrix1x3::from_array([5.0, 6.0, 7.0]);
        assert_eq!(r * c, 56.0);
        let o = c * r;
        assert_eq!(
            o.as_array(),
            &[
                [10.0, 12.0, 14.0],
                [15.0, 18.0, 21.0],
                [20.0, 24.0, 28.0]
            ]
        );
    }

    #[test]
    fn transpose_roundtrip() {
        let c = Matrix3x1::from_array([1.0, 2.0, 3.0]);
        assert_eq!(c.transpose().transpose(), c);
    }
}
