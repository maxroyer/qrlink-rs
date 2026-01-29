use rand::Rng;

/// Base56 alphabet (excludes ambiguous characters like '0', 'O', 'I', 'l', '1', etc.)
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789";

/// Length of generated short codes.
const SHORT_CODE_LENGTH: usize = 7;

/// A short code identifier for a link.
/// Wraps a String to provide type safety and controlled generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortCode(String);

impl ShortCode {
    /// Generate a new random short code.
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let code: String = (0..SHORT_CODE_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..ALPHABET.len());
                ALPHABET[idx] as char
            })
            .collect();
        ShortCode(code)
    }

    /// Create a ShortCode from an existing string (e.g., from database).
    pub fn from_existing(code: String) -> Self {
        ShortCode(code)
    }

    /// Get the short code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ShortCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_code_length() {
        let code = ShortCode::generate();
        assert_eq!(code.as_str().len(), SHORT_CODE_LENGTH);
    }

    #[test]
    fn test_short_code_alphabet() {
        let code = ShortCode::generate();
        for c in code.as_str().chars() {
            assert!(
                ALPHABET.contains(&(c as u8)),
                "Character {} not in alphabet",
                c
            );
        }
    }

    #[test]
    fn test_short_code_no_ambiguous_chars() {
        // Generate many codes to increase confidence
        for _ in 0..100 {
            let code = ShortCode::generate();
            let s = code.as_str();
            assert!(!s.contains('o'));
            assert!(!s.contains('O'));
            assert!(!s.contains('0'));
            assert!(!s.contains('I'));
            assert!(!s.contains('l'));
            assert!(!s.contains('1'));
        }
    }

    #[test]
    fn test_short_codes_are_different() {
        let code1 = ShortCode::generate();
        let code2 = ShortCode::generate();
        // Extremely unlikely to be the same
        assert_ne!(code1, code2);
    }
}
