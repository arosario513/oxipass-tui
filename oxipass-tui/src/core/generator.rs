use rand::RngExt;
use zxcvbn::zxcvbn;

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

pub struct PasswordGen {
    pub password: String,
    pub length: usize,
    pub use_upper: bool,
    pub use_lower: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
}

impl Default for PasswordGen {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordGen {
    pub fn new() -> Self {
        let mut g = Self {
            password: String::new(),
            length: 16,
            use_upper: true,
            use_lower: true,
            use_digits: true,
            use_symbols: true,
        };
        g.regenerate();
        g
    }

    fn charset(&self) -> Vec<u8> {
        let sets = [
            (self.use_lower, LOWER),
            (self.use_upper, UPPER),
            (self.use_digits, DIGITS),
            (self.use_symbols, SYMBOLS),
        ];
        let mut chars: Vec<u8> = sets
            .iter()
            .filter_map(|(enabled, set)| if *enabled { Some(*set) } else { None })
            .flatten()
            .copied()
            .collect();
        if chars.is_empty() {
            chars.extend_from_slice(LOWER);
        }
        chars
    }

    pub fn charset_size(&self) -> usize {
        self.charset().len()
    }

    pub fn entropy_bits(&self) -> f64 {
        let cs = self.charset_size() as f64;
        if cs < 2.0 {
            return 0.0;
        }
        (self.length as f64) * cs.log2()
    }

    pub fn regenerate(&mut self) {
        let charset = self.charset();
        let mut rng = rand::rng();
        self.password = (0..self.length)
            .map(|_| charset[rng.random_range(0..charset.len())] as char)
            .collect();
    }

    pub fn score(&self) -> u8 {
        u8::from(zxcvbn(&self.password, &[]).score())
    }

    pub fn strength_label(&self) -> &'static str {
        match self.score() {
            0 | 1 => "Weak",
            2 => "Moderate",
            3 => "Strong",
            _ => "Very Strong",
        }
    }

    pub fn increase_length(&mut self) {
        if self.length < 64 {
            self.length += 1;
            self.regenerate();
        }
    }

    pub fn decrease_length(&mut self) {
        if self.length > 4 {
            self.length -= 1;
            self.regenerate();
        }
    }
}
