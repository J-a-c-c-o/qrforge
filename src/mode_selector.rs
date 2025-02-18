use crate::{
    constants::ALPHANUMERIC, encode, utils, ErrorCorrection, Mode, QRError, Version
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

pub fn optimize_segments(
    segments: &Vec<(Mode, Vec<u8>)>,
) -> Vec<(Mode, Vec<u8>)> {
    // combine segments of the same mode
    let mut optimized_segments = Vec::new();
    let mut current_mode = Mode::Byte;
    let mut current_data = Vec::new();

    for (mode, data) in segments {
        if mode.to_value() == current_mode.to_value() {
            current_data.extend_from_slice(data);
        } else {
            if !current_data.is_empty() {
                optimized_segments.push((current_mode, current_data));
            }

            current_mode = mode.clone();
            current_data = data.clone();
        }
    }

    if !current_data.is_empty() {
        optimized_segments.push((current_mode, current_data));
    }

    optimized_segments
}
