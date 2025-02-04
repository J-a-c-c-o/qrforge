use std::fmt;
use std::error::Error;

pub struct QRError {
    message: String,
}

impl QRError {
    pub fn new(message: &str) -> QRError {
        QRError {
            message: message.to_string(),
        }
    }
}

impl Error for QRError {
}

impl fmt::Display for QRError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for QRError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}