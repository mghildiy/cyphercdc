use core::fmt;

pub struct Rsi {
    pub nonce: String,
    pub salt: String,
    pub iter_count: u32
}

impl fmt::Display for Rsi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nonce: {}, salt: {}, ier_count:{}", self.nonce, self.salt, self.iter_count)
    }
}