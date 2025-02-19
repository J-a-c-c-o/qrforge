use crate::{constants::DATA_CODEWORDS, ErrorCorrection};

pub(crate) fn get_available_data_size(
    version: usize,
    error_correction: &ErrorCorrection,
) -> u32 {
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

pub(crate) fn is_eci(data: &[bool]) -> bool {
    // check if the first 4 bits are 0111
    data.len() >= 4 && data[0..4] == [false, true, true, true]
}