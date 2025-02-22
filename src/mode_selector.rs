use crate::{constants::ALPHANUMERIC, encode, utils, ErrorCorrection, Mode, QRError, Version};

/// Select the mode for the data
pub(crate) fn select_mode(data: &[u8]) -> Mode {
    if data.iter().all(|&c| c.is_ascii_digit()) {
        Mode::Numeric
    } else if data.iter().all(|&c| ALPHANUMERIC.contains(&(c as char))) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

/// Get the version for the data
pub(crate) fn get_version(
    segments: &Vec<(Mode, Vec<u8>)>,
    error_correction: &ErrorCorrection,
) -> Result<Version, QRError> {
    for i in 1..=40 {
        let mut data_size = 0;
        for (mode, data) in segments {
            let (mode, data) = encode::encode_segment(i - 1, mode, data);
            data_size += mode.len() + data.len();
        }

        let capacity = utils::get_available_data_size(i, error_correction);

        if data_size <= capacity as usize {
            return Ok(Version::V(i));
        }
    }

    Err(QRError::new("Data is too large"))
}
