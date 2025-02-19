mod correction;
mod encode;
mod error;
mod interleave;
mod matrix_builder;
mod matrix_builder_micro;
mod mode_selector;
use error::QRError;
mod constants;
mod image;
mod utils;

use std::collections::VecDeque;

pub struct QRCode {
    matrix: Vec<bool>,
    some_matrix: Vec<bool>,
    dimension: usize,
}

pub struct QRBuilder {
    version: Option<Version>,
    error_correction: Option<ErrorCorrection>,
    segments: Vec<(Mode, Vec<u8>)>,
}

impl QRBuilder {
    pub fn new() -> QRBuilder {
        QRBuilder {
            version: None,
            error_correction: None,
            segments: vec![],
        }
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn error_correction(mut self, ec: ErrorCorrection) -> Self {
        self.error_correction = Some(ec);
        self
    }

    pub fn add_segment(mut self, mode: Option<Mode>, bytes: &[u8]) -> Self {
        if mode.is_some() {
            self.segments.push((mode.unwrap(), bytes.to_vec()));
        } else {
            self.segments
                .push((mode_selector::select_mode(&bytes), bytes.to_vec()));
        }
        self
    }

    pub fn put_eci(mut self, eci: usize) -> Self {
        self.segments.push((Mode::ECI(eci), vec![]));
        self
    }

    pub fn build(self) -> Result<QRCode, QRError> {
        let error_correction = self.error_correction.unwrap_or(ErrorCorrection::M);
        let segments = mode_selector::optimize_segments(&self.segments);

        let version = match self.version {
            Some(v) => v,
            None => mode_selector::get_version(&segments, &error_correction)?,
        };

        QRCode::build(version, error_correction, &segments)
    }

    pub fn build_with_structual_append(self) -> Result<Vec<QRCode>, QRError> {
        let error_correction = self.error_correction.unwrap_or(ErrorCorrection::M);
        let segments = mode_selector::optimize_segments(&self.segments);

        let version = match self.version {
            Some(v) => v,
            None => return Err(QRError::new("Version is required for structured append")),
        };

        QRCode::build_with_structual_append(version, error_correction, &segments)
    }
}

impl QRCode {
    pub fn builder() -> QRBuilder {
        QRBuilder::new()
    }

    pub fn image_builder(&self) -> image::ImageQRCode {
        image::ImageQRCode::new(self.clone())
    }

    fn build(
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

    fn build_with_structual_append(
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
                // split the data
                // calculate how many data we can fit
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

            // add structual append header
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
                // 8 bits
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

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.matrix[y * self.dimension + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.matrix[y * self.dimension + x] = value;
        self.some_matrix[y * self.dimension + x] = true;
    }

    fn is_empty(&self, x: usize, y: usize) -> bool {
        !self.some_matrix[y * self.dimension + x]
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn clone(&self) -> QRCode {
        let matrix = self.matrix.clone();
        let some_matrix = self.some_matrix.clone();
        QRCode {
            matrix,
            some_matrix,
            dimension: self.dimension,
        }
    }

    fn calculate_dimension(version: usize) -> usize {
        if version >= 1 && version <= 40 {
            (version - 1) * 4 + 21
        } else {
            (version - 41) * 2 + 11
        }
    }

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

#[derive(Clone)]
pub enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
    ECI(usize),
}

impl Mode {
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

    pub fn to_value(&self) -> usize {
        match self {
            Mode::Numeric => 0,
            Mode::Alphanumeric => 1,
            Mode::Byte => 2,
            Mode::Kanji => 3,
            Mode::ECI(_) => 4,
        }
    }

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

pub enum ErrorCorrection {
    L,
    M,
    Q,
    H,
}

impl ErrorCorrection {
    pub fn from(value: usize) -> ErrorCorrection {
        match value {
            0 => ErrorCorrection::L,
            1 => ErrorCorrection::M,
            2 => ErrorCorrection::Q,
            3 => ErrorCorrection::H,
            _ => panic!("Invalid error correction level"),
        }
    }

    pub fn to_value(&self) -> usize {
        match self {
            ErrorCorrection::L => 0,
            ErrorCorrection::M => 1,
            ErrorCorrection::Q => 2,
            ErrorCorrection::H => 3,
        }
    }
}

pub enum Version {
    V(usize),
    M(usize),
}

fn main() -> Result<(), QRError> {
    let start = std::time::Instant::now();

    // Japanese text "こんにちは" (Hello) in Shift-JIS encoding;
    let kanji = vec![0x93, 0xfa, 0x96, 0x7b, 0x82, 0xcc, 0x82, 0xc1, 0x82, 0xbd];
    let utf8: Vec<u8> = vec![255, 61];
    let bytes = b"Hello world";
    // 7089 numbers
    let mut numbers = vec![];
    for i in 0..35 {
        numbers.push((i % 10) as u8 + 48);
    }

    // let _simple_qr_code = QRCode::builder()
    //     .add_segment(Some(Mode::Numeric), &numbers)
    //     .add_segment(Some(Mode::Numeric), &numbers)
    //     // .version(40)
    //     .error_correction(ErrorCorrection::L)
    //     .build_with_structual_append(2)?;
    //     // .image_builder()
    //     // .set_width(200)
    //     // .set_height(200)
    //     // .set_border(4)
    //     // .build_svg_file("hello.svg")?;

    // let _qr_code = QRCode::builder()
    //     .add_segment(Some(Mode::Kanji), &kanji)
    //     .add_segment(None, bytes)
    //     .add_segment(Some(Mode::Alphanumeric), b"HELLO ")
    //     .add_segment(Some(Mode::Numeric), &numbers)
    //     .add_segment(Some(Mode::ECI(3)), &utf8)
    //     .error_correction(ErrorCorrection::L)
    //     .version(Version::V(5))
    //     .build()?
    //     .image_builder()
    //     .set_width(200)
    //     .set_height(200)
    //     .set_border(4)
    //     .build_image_file("hello_0.png")?;

    let structured = QRCode::builder()
        .add_segment(Some(Mode::Byte), b"I read the newspaper")
        .add_segment(Some(Mode::Numeric), b"1234567890")
        .error_correction(ErrorCorrection::L)
        .version(Version::V(1))
        .build_with_structual_append()?;

    for (index, qr_code) in structured.iter().enumerate() {
        qr_code
            .image_builder()
            .set_width(200)
            .set_height(200)
            .set_border(4)
            .build_image_file(&format!("hello_{}.png", index))?;
    }
    println!("QR Code generated in: {:?}", start.elapsed());

    Ok(())
}
