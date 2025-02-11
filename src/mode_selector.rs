use crate::{
    constants::{ALPHANUMERIC, DATA_CODEWORDS},
    encode, ErrorCorrection, Mode, QRError, Version,
};

pub(crate) fn select_mode(data: &[u8]) -> Mode {
    if data.iter().all(|&c| c >= b'0' && c <= b'9') {
        Mode::Numeric
    } else if data.iter().all(|&c| ALPHANUMERIC.contains(&(c as char))) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

pub(crate) fn get_version(
    segments: &Vec<(Mode, Vec<u8>)>,
    error_correction: &ErrorCorrection,
    structual_append: Option<usize>,
) -> Result<Version, QRError> {
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

        let capacity = DATA_CODEWORDS[i][error_correction.to_value()] * 8;

        if data_size <= capacity as usize {
            return Ok(Version::V(i + 1));
        }
    }

    Err(QRError::new("Data is too large"))
}
