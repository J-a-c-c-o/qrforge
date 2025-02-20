use std::error::Error;
use std::fmt;

/// Error type for QR code generation
///
/// # Example
///
/// ``` rust
///
/// # use qrcode::QRError;
///
/// fn main() -> Result<(), QRError> {
///    Err(QRError::new("An error occurred"))
/// }
/// ```
pub struct QRError {
    message: String,
}

impl QRError {
    /// Create a new error
    pub fn new(message: &str) -> QRError {
        QRError {
            message: message.to_string(),
        }
    }
}

impl Error for QRError {}

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
