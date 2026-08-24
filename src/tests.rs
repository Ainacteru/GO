pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}