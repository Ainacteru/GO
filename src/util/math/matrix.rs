pub mod matrix2x2;
pub mod matrix3x3;

use crate::util::math::error::MatrixError;

pub trait Matrix<T, const R: usize, const C: usize>: Sized {
    fn new() -> Self;
    fn from_array(array: [[T; R]; C]) -> Self;
    fn new_diagonal(diagonal: [T; C]) -> Self;

    fn get_row_col(&self) -> (usize, usize) {
        (R, C)
    }

    fn transpose(&self) -> Self;
    fn scale(&self, rhs: f32) -> Self;
    fn determinant(&self) -> f32;
    fn cofactor(&self) -> Self;
    fn adjugate(&self) -> Self;
    fn inverse(&self) -> Result<Self, MatrixError>;
}
