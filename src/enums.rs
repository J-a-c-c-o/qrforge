/// Represents the various encoding modes available for QR codes.
#[derive(Clone)]
pub enum Mode {
    /// Numeric mode: encodes digits 0-9.
    Numeric,
    /// Alphanumeric mode: encodes digits, uppercase letters and a few symbols.
    Alphanumeric,
    /// Byte mode: encodes characters using 8-bit bytes.
    Byte,
    /// Kanji mode: encodes characters using Shift JIS.
    Kanji,
    /// Extended Channel Interpretation mode.
    ECI(usize),
}

impl Mode {
    /// Creates a `Mode` from a numeric value.
    ///
    /// # Panics
    ///
    /// Panics if the value does not represent a valid mode.
    pub fn from(value: usize) -> Mode {
        match value {
            0 => Mode::Numeric,
            1 => Mode::Alphanumeric,
            2 => Mode::Byte,
            3 => Mode::Kanji,
            4 => Mode::ECI(0),
            _ => panic!("Invalid mode"),
        }
    }

    /// Returns a numeric representation of the mode.
    pub fn to_value(&self) -> usize {
        match self {
            Mode::Numeric => 0,
            Mode::Alphanumeric => 1,
            Mode::Byte => 2,
            Mode::Kanji => 3,
            Mode::ECI(_) => 4,
        }
    }

    /// Creates a clone of the `Mode`.
    pub fn clone(&self) -> Mode {
        match self {
            Mode::Numeric => Mode::Numeric,
            Mode::Alphanumeric => Mode::Alphanumeric,
            Mode::Byte => Mode::Byte,
            Mode::Kanji => Mode::Kanji,
            Mode::ECI(v) => Mode::ECI(*v),
        }
    }
}

/// Represents the error correction levels available for QR codes.
pub enum ErrorCorrection {
    /// Low error correction.
    L,
    /// Medium error correction.
    M,
    /// Quality error correction.
    Q,
    /// High error correction.
    H,
}

impl ErrorCorrection {
    /// Creates an `ErrorCorrection` level from a numeric value.
    ///
    /// # Panics
    ///
    /// Panics if the value does not represent a valid error correction level.
    pub fn from(value: usize) -> ErrorCorrection {
        match value {
            0 => ErrorCorrection::L,
            1 => ErrorCorrection::M,
            2 => ErrorCorrection::Q,
            3 => ErrorCorrection::H,
            _ => panic!("Invalid error correction level"),
        }
    }

    /// Returns a numeric representation of the error correction level.
    pub fn to_value(&self) -> usize {
        match self {
            ErrorCorrection::L => 0,
            ErrorCorrection::M => 1,
            ErrorCorrection::Q => 2,
            ErrorCorrection::H => 3,
        }
    }
}

/// Represents the QR code version.
///
/// `Version` distinguishes standard QR codes (V) from micro QR codes (M).
pub enum Version {
    /// Standard QR code version.
    V(usize),
    /// Micro QR code version.
    M(usize),
}
