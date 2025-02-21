pub fn mod_exp(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    };

    let mut result = 1;
    base = base % modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }
        exp = exp >> 1;
        base = (base * base) % modulus;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_exp() {
        // Existing tests
        assert_eq!(mod_exp(0, 10, 1), 0); // Any number mod 1 is 0
        assert_eq!(mod_exp(0, 0, 5), 1); // By definition, 0^0 = 1 in modular arithmetic
        assert_eq!(mod_exp(5, 3, 13), 8); // Basic test

        assert_eq!(mod_exp(2, 0, 7), 1); // x^0 mod m = 1
        assert_eq!(mod_exp(1, 1000, 2), 1); // 1 to any power mod m = 1
        assert_eq!(mod_exp(10, 1, 2), 0); // Even number to odd power mod 2 = 0
        assert_eq!(mod_exp(2, 10, 1024), 1024 % 1024); // 2^10 = 1024

        // Edge cases
        assert_eq!(mod_exp(0, 0, 1), 0); // Special case with mod 1
        assert_eq!(mod_exp(100, 0, 1), 0); // Any base with exponent 0 mod 1
        assert_eq!(mod_exp(0, 5, 3), 0); // 0^n mod m = 0 (for n > 0)

        // Large base and modulus
        assert_eq!(mod_exp(987654321, 1, 41242144), 39085009);
    }
}
