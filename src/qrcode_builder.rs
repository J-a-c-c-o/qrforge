use crate::{mode_selector, qrcode::QRCode, utils, ErrorCorrection, Mode, QRError, Version};

/// A builder to create a QRCode.
///
/// The QRBuilder supports setting parameters such as version, error correction level,
/// and adding data segments. It also offers two build modes: a standard build and one
/// using structured append that splits the data across multiple QR codes.
///
/// # Examples
///
/// Creating a QR code using the builder:
///
/// ```rust
/// let qr = QRCode::builder()
///     .add_segment(Some(Mode::Byte), b"I read the newspaper")
///     .add_segment(Some(Mode::Numeric), b"1234567890")
///     .error_correction(ErrorCorrection::L)
///     .version(Version::V(1))
///     .build()?;
/// ```
///
/// Creating a structured append QR code:
///
/// ```rust
/// let qr_codes = QRCode::builder()
///     .add_segment(Some(Mode::Byte), b"I read the newspaper")
///     .add_segment(Some(Mode::Numeric), b"1234567890")
///     .error_correction(ErrorCorrection::L)
///     .version(Version::V(1))
///     .build_with_structual_append()?;
/// ```
pub struct QRBuilder {
    version: Option<Version>,
    error_correction: Option<ErrorCorrection>,
    segments: Vec<(Mode, Vec<u8>)>,
}

impl QRBuilder {
    /// Creates a new empty QRBuilder.
    pub fn new() -> QRBuilder {
        QRBuilder {
            version: None,
            error_correction: None,
            segments: vec![],
        }
    }

    /// Sets the version for the QR code.
    ///
    /// The version determines the size of the QR code. For standard QR codes, valid versions
    /// are between 1 and 40. For micro QR codes, valid versions are between 1 and 5.
    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the error correction level.
    ///
    /// The error correction level determines the ability of the QR code to recover data in case
    /// of damage or errors. Higher levels provide better recovery at the cost of reduced data capacity.
    pub fn error_correction(mut self, ec: ErrorCorrection) -> Self {
        self.error_correction = Some(ec);
        self
    }

    /// Adds a data segment to the QR code.
    ///
    /// If `mode` is provided, the segment is encoded with that mode; otherwise,
    /// the mode is automatically selected based on the bytes provided.
    pub fn add_segment(mut self, mode: Option<Mode>, bytes: &[u8]) -> Self {
        if let Some(m) = mode {
            self.segments.push((m, bytes.to_vec()));
        } else {
            self.segments
                .push((mode_selector::select_mode(&bytes), bytes.to_vec()));
        }
        self
    }

    /// Adds an Extended Channel Interpretation (ECI) segment.
    ///
    /// This method inserts an ECI mode into the segments with the given identifier.
    pub fn put_eci(mut self, eci: usize) -> Self {
        self.segments.push((Mode::ECI(eci), vec![]));
        self
    }

    /// Builds a QR code using the segments and parameters provided.
    ///
    /// If no version is provided, the version is determined automatically based on the data.
    ///
    /// # Errors
    ///
    /// Returns a `QRError` if an error occurs during the building process.
    pub fn build(self) -> Result<QRCode, QRError> {
        let error_correction = self.error_correction.unwrap_or(ErrorCorrection::M);
        let segments = utils::optimize_segments(&self.segments);

        let version = match self.version {
            Some(v) => v,
            None => mode_selector::get_version(&segments, &error_correction)?,
        };

        QRCode::build(version, error_correction, &segments)
    }

    /// Builds QR codes using structured append.
    ///
    /// This method splits the input data into multiple QR codes.
    ///
    /// # Errors
    ///
    /// Returns a `QRError` if the version is not provided or if the building process fails.
    pub fn build_with_structual_append(self) -> Result<Vec<QRCode>, QRError> {
        let error_correction = self.error_correction.unwrap_or(ErrorCorrection::M);
        let segments = utils::optimize_segments(&self.segments);

        let version = match self.version {
            Some(v) => v,
            None => return Err(QRError::new("Version is required for structured append")),
        };

        QRCode::build_with_structual_append(version, error_correction, &segments)
    }
}
