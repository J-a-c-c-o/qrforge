mod correction;
mod encode;
mod error;
mod interleave;
mod matrix_builder;
mod mode_selector;
use error::QRError;
mod image;

pub struct QRCode {
    matrix: Vec<bool>,
    some_matrix: Vec<bool>,
    dimension: usize,
}

pub struct QRBuilder {
    version: Option<usize>,
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

    pub fn version(mut self, version: usize) -> Self {
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
            self.segments.push((mode_selector::select_mode(&bytes), bytes.to_vec()));
            // self.segments.push((Mode::Byte, bytes.to_vec()));
        }
        self
    }


    pub fn build(self) -> Result<QRCode, QRError> {
        let error_correction = self.error_correction.unwrap_or(ErrorCorrection::M);

        let version = match self.version {
            Some(v) => v,
            // None => mode_selector::get_version(&self.segments, &error_correction)?,
            None => 5,
        };

        QRCode::build(version, error_correction, &self.segments)
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
        version: usize,
        error_correction: ErrorCorrection,
        segments: &[(Mode, Vec<u8>)],
    ) -> Result<QRCode, QRError> {
        let dimension = Self::calculate_dimension(version);


        let mut matrix = QRCode {
            matrix: vec![false; dimension * dimension],
            some_matrix: vec![false; dimension * dimension],
            dimension,
        };

        let mut combined_data = vec![];

        for (mode, bytes) in segments {
            match mode {
                Mode::ECI(_) => {
                    combined_data.extend_from_slice(&encode::encode_segment(version, mode, &[]));
                    combined_data.extend_from_slice(&encode::encode_segment(version, &Mode::Byte, bytes));
                }
                _ => {
                    combined_data.extend_from_slice(&encode::encode_segment(version, mode, bytes));
                }
            }
        }

        let combined_data = encode::build_combined_data(combined_data, version, &error_correction)?;

    
        
        let (blocks, ec_blocks) = correction::correction(version, &error_correction, combined_data);
        let result = interleave::interleave(blocks, ec_blocks, version);

        matrix_builder::build_qr_matrix(&mut matrix, version, &error_correction, result);

        Ok(matrix)
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
        (version - 1) * 4 + 21
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

fn main() -> Result<(), QRError> {
    let start = std::time::Instant::now();

    // Japanese text "こんにちは" (Hello) in Shift-JIS encoding;
    let kanji = vec![0x93, 0xfa, 0x96, 0x7b, 0x82, 0xcc, 0x82, 0xc1, 0x82, 0xbd];
    let utf8: Vec<u8> = vec![255, 61];
    let bytes = b" Hello";

    let _qr_code = QRCode::builder()
        .add_segment(Some(Mode::Kanji), &kanji)
        .add_segment(None, bytes)
        .add_segment(Some(Mode::Alphanumeric), b"HELLO ")
        .add_segment(Some(Mode::Numeric), b"123456")
        .add_segment(Some(Mode::ECI(3)), &utf8)
        .error_correction(ErrorCorrection::H)
        .version(4)
        .build()?
        .image_builder()
        .set_width(200)
        .set_height(200)
        .set_border(4)
        .build_svg_file("hello_japanese.svg")?;

    println!("QR Code generated in: {:?}", start.elapsed());

    Ok(())
}
