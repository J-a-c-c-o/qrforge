use crate::{encode, ErrorCorrection, Mode, QRError};

pub(crate) fn select_mode(data: &[u8]) -> Mode {
    if data.iter().all(|&c| c >= b'0' && c <= b'9') {
        Mode::Numeric
    } else if data.iter().all(|&c| ALPHANUMERIC.contains(&(c as char))) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

const ALPHANUMERIC: [char; 45] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ', '$',
    '%', '*', '+', '-', '.', '/', ':',
];

pub(crate) fn get_version(
    segments: &Vec<(Mode, Vec<u8>)>,
    error_correction: &ErrorCorrection,
    structual_append: Option<usize>,
) -> Result<usize, QRError> {
    for i in 0..40 {
        let mut data_size = 0;
        for (mode, data) in segments {
            let encoded = encode::encode_segment(i, mode, data);
            data_size += encoded.len();
        }

        if let Some(structual_append) = structual_append {
            // devide it by structural_append to get the size of each qr code if uneven round up
            if data_size % 2 == 1 {
                data_size += 1;
            }

            data_size /= structual_append;

            data_size += 20; // 20 bits for the mode indicator and character count indicator
        }

        let capacity = crate::encode::DATA_CODEWORDS[i][error_correction.to_value()] * 8;

        if data_size <= capacity as usize {
            return Ok(i + 1);
        }
    }

    Err(QRError::new("Data is too large"))
}


