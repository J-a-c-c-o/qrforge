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