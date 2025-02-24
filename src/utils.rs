use crate::{constants::DATA_CODEWORDS, ErrorCorrection, Mode};

/// Get the number of data codewords for a given version and error correction level
pub(crate) fn get_available_data_size(version: usize, error_correction: &ErrorCorrection) -> u32 {
    if version > 44 {
        panic!("Invalid version");
    }

    DATA_CODEWORDS[version - 1][error_correction.to_value()]
}

/// Convert bits to bytes
pub(crate) fn bits_to_bytes(data: &[bool]) -> Vec<u8> {
    let mut bytes = vec![];
    let mut byte = 0;

    for (i, bit) in data.iter().enumerate() {
        byte |= (*bit as u8) << (7 - i % 8);

        if i % 8 == 7 {
            bytes.push(byte);
            byte = 0;
        }
    }

    bytes
}

/// calculate the number of bits it takes to encode the data
pub(crate) fn num_of_bits(mode: &Mode, bytes: usize) -> usize {
    match mode {
        Mode::Numeric => match bytes % 3 {
            0 => bytes / 3 * 10,
            1 => (bytes / 3) * 10 + 4,
            2 => (bytes / 3) * 10 + 7,
            _ => panic!("Invalid number of bytes"),
        },

        Mode::Alphanumeric => match bytes % 2 {
            0 => bytes / 2 * 11,
            1 => (bytes / 2) * 11 + 6,
            _ => panic!("Invalid number of bytes"),
        },

        Mode::Byte => bytes * 8,

        Mode::Kanji => bytes * 13,

        Mode::ECI(_) => 0,
    }
}

/// Optimize the segments, combining segments of the same mode
pub(crate) fn optimize_segments(segments: &Vec<(Mode, Vec<u8>)>) -> Vec<(Mode, Vec<u8>)> {
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
