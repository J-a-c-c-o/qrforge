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
    bytes: Vec<u8>,
    version: Option<usize>,
    error_correction: Option<ErrorCorrection>,
    mode: Option<Mode>,
}

impl QRBuilder {
    pub fn new(bytes: &[u8]) -> QRBuilder {
        QRBuilder {
            bytes: bytes.to_vec(),
            version: None,
            error_correction: None,
            mode: None,
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

    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn build(self) -> Result<QRCode, QRError> {
        let error_correction = self.error_correction.unwrap_or(ErrorCorrection::M);
        let mode = self
            .mode
            .unwrap_or_else(|| mode_selector::select_mode(&self.bytes));
        let version = match self.version {
            Some(v) => v,
            None => mode_selector::get_version(&self.bytes, &error_correction, &mode)?,
        };

        QRCode::build(version, error_correction, mode, &self.bytes)
    }
}

impl QRCode {
    pub fn builder(bytes: &[u8]) -> QRBuilder {
        QRBuilder::new(bytes)
    }

    pub fn image_builder(&self) -> image::ImageQRCode {
        image::ImageQRCode::new(self.clone())
    }

    fn build(
        version: usize,
        error_correction: ErrorCorrection,
        mode: Mode,
        bytes: &[u8],
    ) -> Result<QRCode, QRError> {
        let dimension = Self::calculate_dimension(version);

        let capacity = match mode {
            Mode::Numeric => bytes.len() * 3 / 10,
            Mode::Alphanumeric => bytes.len() * 2 / 5,
            Mode::Byte => bytes.len(),
            Mode::Kanji => bytes.len() / 2,
        };

        if capacity > mode_selector::get_capacity(version, &error_correction, &mode) {
            return Err(QRError::new("Data is too long"));
        }

        let mut matrix = QRCode {
            matrix: vec![false; dimension * dimension],
            some_matrix: vec![false; dimension * dimension],
            dimension,
        };

        let combined_data = encode::encode(version, &error_correction, &mode, bytes);
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

enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
}

impl Mode {
    pub fn from(value: usize) -> Mode {
        match value {
            0 => Mode::Numeric,
            1 => Mode::Alphanumeric,
            2 => Mode::Byte,
            3 => Mode::Kanji,
            _ => panic!("Invalid mode"),
        }
    }

    pub fn to_value(&self) -> usize {
        match self {
            Mode::Numeric => 0,
            Mode::Alphanumeric => 1,
            Mode::Byte => 2,
            Mode::Kanji => 3,
        }
    }
}

enum ErrorCorrection {
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

    let qr_code = QRCode::builder(&kanji)
        .error_correction(ErrorCorrection::H)
        .mode(Mode::Kanji)
        .version(5)
        .build()?
        .image_builder()
        .set_width(200)
        .set_height(200)
        .set_border(4)
        .build_svg_file("hello_japanese.svg")?;

    println!("QR Code generated in: {:?}", start.elapsed());

    Ok(())
}
