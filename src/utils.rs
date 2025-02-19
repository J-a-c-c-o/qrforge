use crate::{constants::DATA_CODEWORDS, ErrorCorrection, Mode};

pub(crate) fn get_available_data_size(version: usize, error_correction: &ErrorCorrection) -> u32 {
    if version > 44 {
        panic!("Invalid version");
    }

    DATA_CODEWORDS[version - 1][error_correction.to_value()] * if version <= 40 { 8 } else { 0 }
}

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
