mod encode;
mod correction;
mod matrix_builder;
mod interleave;

pub struct QRCode {
    matrix: Vec<bool>,
    some_matrix: Vec<bool>,
    dimension: usize,
}

impl QRCode {
    fn new(version: usize, error_correction: ErrorCorrection, mode: Mode, text: &str) -> QRCode {
        let dimension = Self::get_dimension(version);

        let mut matrix = QRCode {
            matrix: vec![false; dimension * dimension],
            some_matrix: vec![false; dimension * dimension],
            dimension,
        };

        let combined_data = encode::encode(version, &error_correction, &mode, text);
        let (blocks, ec_blocks) = correction::correction(version, &error_correction, combined_data);
        let result = interleave::interleave(blocks, ec_blocks, version);

        matrix_builder::build_qr_matrix(&mut matrix, version, &error_correction, result);

        matrix
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.matrix[y * self.dimension + x]
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        self.matrix[y * self.dimension + x] = value;
        self.some_matrix[y * self.dimension + x] = true;
    }

    fn is_empty(&self, x: usize, y: usize) -> bool {
        !self.some_matrix[y * self.dimension + x]
    }
    
    fn len(&self) -> usize {
        self.dimension
    }

    fn clone(&self) -> QRCode {
        let matrix = self.matrix.clone();
        let some_matrix = self.some_matrix.clone();
        QRCode {
            matrix,
            some_matrix,
            dimension: self.dimension,
        }
    }

    fn get_dimension(version: usize) -> usize {
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

enum ErrorCorrection {
    L,
    M,
    Q,
    H,
}

fn main() {
    let version = 40;
    let error_correction = ErrorCorrection::H;
    let mode = Mode::Byte;
    let text = "HELLO WORLD";
      
    let start = std::time::Instant::now();
    let matrix = QRCode::new(version, error_correction, mode, text);
    println!("Time: {:?}", start.elapsed());

    // matrix.print();   
    
}

