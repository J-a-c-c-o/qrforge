use std::collections::VecDeque;

use crate::{
    correction, encode, interleave, matrix_builder, matrix_builder_micro,
    qrcode_builder::QRBuilder, utils, ErrorCorrection, Mode, QRError, Version,
};

#[cfg(feature = "image")]
use crate::image;

#[cfg(feature = "svg")]
use crate::svg;

/// Represents a QR code matrix.
///
/// The QRCode holds a two-dimensional matrix of booleans indicating the QR code
/// pattern as well as an auxiliary matrix used during the matrix building process.
/// The `dimension` field represents the width/height of the matrix.
pub struct QRCode {
    matrix: Vec<bool>,
    some_matrix: Vec<bool>,
    dimension: usize,
}

impl QRCode {
    /// Returns a new builder to construct a QRCode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = QRCode::builder();
    /// ```
    pub fn builder() -> QRBuilder {
        QRBuilder::new()
    }

    /// Returns an image builder for the QR code.
    ///
    /// The image builder can be used to create a visual representation (PNG, SVG, etc.)
    /// of the QR code matrix.
    #[cfg(feature = "image")]
    pub fn image_builder(&self) -> image::ImageQRCode {
        image::ImageQRCode::new(self.clone())
    }

    /// Returns an svg builder for the QR code.
    ///
    /// The svg builder can be used to create a visual representation of the QR code matrix in SVG format.
    #[cfg(feature = "svg")]
    pub fn svg_builder(&self) -> svg::SvgQRCode {
        svg::SvgQRCode::new(self.clone())
    }

    /// Internal method to build a QR code.
    ///
    /// This function encodes the segments, applies error correction, interleaves data,
    /// and then builds the QR matrix using either the standard or micro method based on the version.
    ///
    /// # Errors
    ///
    /// Returns a `QRError` if the version is invalid or if there is an error during encoding.
    pub(crate) fn build(
        version: Version,
        error_correction: ErrorCorrection,
        segments: &[(Mode, Vec<u8>)],
    ) -> Result<QRCode, QRError> {
        match version {
            Version::V(v) => {
                if v < 1 || v > 40 {
                    return Err(QRError::new("Invalid version"));
                }
            }
            Version::M(v) => {
                if v < 1 || v > 5 {
                    return Err(QRError::new("Invalid version"));
                }
            }
        }

        let version = match version {
            Version::V(v) => v,
            Version::M(v) => v + 40,
        };

        let dimension = Self::calculate_dimension(version);

        let mut matrix = QRCode {
            matrix: vec![false; dimension * dimension],
            some_matrix: vec![false; dimension * dimension],
            dimension,
        };

        let mut combined_data = vec![];

        for (mode, bytes) in segments {
            let (mode, data) = encode::encode_segment(version, mode, bytes);
            combined_data.extend_from_slice(&mode);
            combined_data.extend_from_slice(&data);
        }

        let combined_data = encode::build_combined_data(combined_data, version, &error_correction)?;

        let (blocks, ec_blocks) = correction::correction(version, &error_correction, combined_data);
        let result = interleave::interleave(blocks, ec_blocks, version);

        match version {
            1..=40 => {
                matrix_builder::build_qr_matrix(&mut matrix, version, &error_correction, result)
            }
            41..=44 => matrix_builder_micro::build_qr_matrix(
                &mut matrix,
                version,
                &error_correction,
                result,
            ),
            _ => return Err(QRError::new("Invalid version")),
        };

        Ok(matrix)
    }

    /// Internal method to build a QR code with structured append.
    ///
    /// Structured append allows one to encode data across multiple QR codes.
    /// It splits the data into chunks and adds a header indicating the chunk order.
    ///
    /// # Errors
    ///
    /// Returns a `QRError` if the version is invalid or if no segments are provided.
    pub(crate) fn build_with_structual_append(
        version: Version,
        error_correction: ErrorCorrection,
        segments: &[(Mode, Vec<u8>)],
    ) -> Result<Vec<QRCode>, QRError> {
        match version {
            Version::V(v) => {
                if v < 1 || v > 40 {
                    return Err(QRError::new("Invalid version"));
                }
            }
            Version::M(_) => {
                return Err(QRError::new(
                    "Structured append is not supported for micro QR codes",
                ));
            }
        }

        let version = match version {
            Version::V(v) => v,
            Version::M(v) => v + 40,
        };

        let dimension = Self::calculate_dimension(version);

        if segments.is_empty() {
            return Err(QRError::new("No segments provided"));
        }

        let max_size = utils::get_available_data_size(version, &error_correction) as usize - 20;

        let mut chunks = vec![];
        let mut current_chunk = vec![];
        let mut current_size = 0;
        let mut eci = None;

        let mut parity = 0;

        let mut mutable_segments: VecDeque<(Mode, Vec<u8>)> = segments.to_vec().into();

        while let Some((mode, data)) = mutable_segments.pop_front() {
            let (mode_b, data_b) = encode::encode_segment(version, &mode, &data);

            match mode {
                Mode::ECI(_) => {
                    eci = Some(mode_b);
                    continue;
                }
                _ => {}
            }

            let size = mode_b.len() + data_b.len();

            if current_size + mode_b.len() >= max_size {
                chunks.push(current_chunk);
                current_chunk = vec![];
                if let Some(eci) = eci.clone() {
                    current_chunk.extend_from_slice(&eci);
                }
                current_size = current_chunk.len();

                mutable_segments.insert(0, (mode, data));
            } else if current_size + size > max_size {
                // split the data into two chunks if it exceeds the maximum size
                let mut left_data = vec![];
                let mut left_size = mode_b.len();

                let mut right_data = vec![];
                let chunk_size = match mode {
                    Mode::Kanji => 2,
                    Mode::Numeric => 3,
                    Mode::Alphanumeric => 2,
                    Mode::Byte => 1,
                    _ => 1,
                };

                for words in data.chunks(chunk_size) {
                    let num_of_bits = utils::num_of_bits(&mode, words.len());

                    if left_size + num_of_bits <= max_size {
                        left_data.extend_from_slice(words);
                        left_size += num_of_bits;
                    } else {
                        right_data.extend_from_slice(words);
                    }
                }

                let (left_mode, left_data) = encode::encode_segment(version, &mode, &left_data);

                current_chunk.extend_from_slice(&left_mode);
                current_chunk.extend_from_slice(&left_data);

                chunks.push(current_chunk);

                current_chunk = vec![];

                if let Some(eci) = eci.clone() {
                    current_chunk.extend_from_slice(&eci);
                }

                current_size = current_chunk.len();

                mutable_segments.insert(0, (mode, right_data));
            } else {
                current_chunk.extend_from_slice(&mode_b);
                current_chunk.extend_from_slice(&data_b);

                for byte in utils::bits_to_bytes(&data_b) {
                    parity ^= byte;
                }

                current_size += size;
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        let mut qr_codes = vec![];

        for (index, segments) in chunks.iter().enumerate() {
            let mut matrix = QRCode {
                matrix: vec![false; dimension * dimension],
                some_matrix: vec![false; dimension * dimension],
                dimension,
            };

            let mut combined_data = vec![];

            // add structured append header
            let mode = [false, false, true, true];
            let index_bits = [
                (index >> 3) & 1 == 1,
                (index >> 2) & 1 == 1,
                (index >> 1) & 1 == 1,
                (index >> 0) & 1 == 1,
            ];

            let total_bits = [
                (chunks.len() >> 3) & 1 == 1,
                (chunks.len() >> 2) & 1 == 1,
                (chunks.len() >> 1) & 1 == 1,
                (chunks.len() >> 0) & 1 == 1,
            ];

            let parity_bits = [
                (parity >> 7) & 1 == 1,
                (parity >> 6) & 1 == 1,
                (parity >> 5) & 1 == 1,
                (parity >> 4) & 1 == 1,
                (parity >> 3) & 1 == 1,
                (parity >> 2) & 1 == 1,
                (parity >> 1) & 1 == 1,
                (parity >> 0) & 1 == 1,
            ];

            combined_data.extend_from_slice(&mode);
            combined_data.extend_from_slice(&index_bits);
            combined_data.extend_from_slice(&total_bits);
            combined_data.extend_from_slice(&parity_bits);

            combined_data.extend_from_slice(&segments);

            let maybe_combined_data =
                encode::build_combined_data(combined_data, version, &error_correction);

            let combined_data = match maybe_combined_data {
                Ok(data) => data,
                Err(e) => return Err(QRError::new(&format!("{} for chunk {}, split the data into smaller chunks using QRBuilder::add_segment", e, index))),
            };

            let (blocks, ec_blocks) =
                correction::correction(version, &error_correction, combined_data);

            let result = interleave::interleave(blocks, ec_blocks, version);

            match version {
                1..=40 => {
                    matrix_builder::build_qr_matrix(&mut matrix, version, &error_correction, result)
                }
                41..=44 => matrix_builder_micro::build_qr_matrix(
                    &mut matrix,
                    version,
                    &error_correction,
                    result,
                ),
                _ => return Err(QRError::new("Invalid version")),
            };

            qr_codes.push(matrix);
        }

        Ok(qr_codes)
    }

    /// Retrieves the value of the QR code module at position (x, y).
    ///
    /// Returns `true` if the module is set (black), otherwise `false` (white).
    pub(crate) fn get(&self, x: usize, y: usize) -> bool {
        self.matrix[y * self.dimension + x]
    }

    /// Sets the value of the QR code module at position (x, y).
    ///
    /// In addition to setting the module's value, this function marks the corresponding
    /// position in the auxiliary matrix as filled.
    pub(crate) fn set(&mut self, x: usize, y: usize, value: bool) {
        self.matrix[y * self.dimension + x] = value;
        self.some_matrix[y * self.dimension + x] = true;
    }

    /// Checks if the module at the given position is empty.
    pub(crate) fn is_empty(&self, x: usize, y: usize) -> bool {
        !self.some_matrix[y * self.dimension + x]
    }

    /// Returns the dimension (width/height) of the QR code.
    pub(crate) fn dimension(&self) -> usize {
        self.dimension
    }

    /// Creates a deep clone of the QRCode.
    pub fn clone(&self) -> QRCode {
        let matrix = self.matrix.clone();
        let some_matrix = self.some_matrix.clone();
        QRCode {
            matrix,
            some_matrix,
            dimension: self.dimension,
        }
    }

    /// Calculates the dimension of the QR code for the given version.
    ///
    /// For standard QR codes (versions 1-40), the dimension is calculated as:
    /// (version - 1) * 4 + 21.
    /// For micro QR codes, a different formula applies.
    fn calculate_dimension(version: usize) -> usize {
        if version >= 1 && version <= 40 {
            (version - 1) * 4 + 21
        } else {
            (version - 41) * 2 + 11
        }
    }

    /// Prints the QR code to the console using ASCII characters.
    ///
    /// Uses "██" for black modules and ("  ") for white modules.
    pub fn print(&self) {
        let black = "██";
        let white = "  ";
        for i in 0..self.dimension {
            for j in 0..self.dimension {
                if self.is_empty(j, i) {
                    print!("{}", "  ");
                } else {
                    print!("{}", if self.get(j, i) { black } else { white });
                }
            }
            println!();
        }
        println!();
    }
}
