use crate::{
    constants::{DATA_CODEWORDS, REMAINDER_BITS},
    error::QRError,
    ErrorCorrection, Mode,
};

pub(crate) fn encode_segment(version: usize, mode: &Mode, bytes: &[u8]) -> Vec<bool> {
    match mode {
        Mode::ECI(_) => {
            let eci_bit_count = get_bit_count(version, mode);
            let eci_mode_indicator = get_mode(mode, version);
            let eci_size = get_size(bytes, eci_bit_count, mode);

            let bit_count = get_bit_count(version, &Mode::Byte);
            let mode_indicator = get_mode(&Mode::Byte, version);
            let size = get_size(bytes, bit_count, &Mode::Byte);
            let data = get_data(bytes, &Mode::Byte);

            let mut combined = vec![];
            combined.extend_from_slice(&eci_mode_indicator);
            combined.extend_from_slice(&eci_size);

            combined.extend_from_slice(&mode_indicator);
            combined.extend_from_slice(&size);
            combined.extend_from_slice(&data);

            combined
        }

        _ => {
            let bit_count = get_bit_count(version, mode);
            let mode_indicator = get_mode(mode, version);
            let size = get_size(bytes, bit_count, mode);
            let data = get_data(bytes, mode);
            build_segment(mode_indicator, size, data)
        }
    }
}

pub(crate) fn encode_structured_append(
    version: usize,
    mode: &Mode,
    error_correction: &ErrorCorrection,
    index: usize,
    total: usize,
    bytes: &[u8],
    parity: u8,
) -> Result<Vec<bool>, QRError> {
    let data = encode_segment(version, mode, bytes);

    // now add the structured append header
    let mut structured_append = vec![];
    let mode_indicator: [bool; 4] = [false, false, true, true];
    let index_bits: [bool; 4] = [
        (index >> 3) & 1 == 1,
        (index >> 2) & 1 == 1,
        (index >> 1) & 1 == 1,
        index & 1 == 1,
    ];

    let total_bits: [bool; 4] = [
        (total >> 3) & 1 == 1,
        (total >> 2) & 1 == 1,
        (total >> 1) & 1 == 1,
        total & 1 == 1,
    ];

    let parity_bits: [bool; 8] = [
        (parity >> 7) & 1 == 1,
        (parity >> 6) & 1 == 1,
        (parity >> 5) & 1 == 1,
        (parity >> 4) & 1 == 1,
        (parity >> 3) & 1 == 1,
        (parity >> 2) & 1 == 1,
        (parity >> 1) & 1 == 1,
        parity & 1 == 1,
    ];

    structured_append.extend_from_slice(&mode_indicator);
    structured_append.extend_from_slice(&index_bits);
    structured_append.extend_from_slice(&total_bits);
    structured_append.extend_from_slice(&parity_bits);
    structured_append.extend_from_slice(&data);

    build_combined_data(structured_append, version, error_correction)
}

pub(crate) fn build_combined_data(
    data: Vec<bool>,
    version: usize,
    error_correction: &ErrorCorrection,
) -> Result<Vec<bool>, QRError> {
    let mut combined_data = vec![];

    let data_codewords =
        lookup_data_codewords(version, error_correction) * if version <= 40 { 8 } else { 1 };

    if data_codewords == 0 {
        return Err(QRError::new("Invalid version"));
    }

    if data.len() > data_codewords as usize {
        return Err(QRError::new("Data too large"));
    }

    // Add data
    combined_data.extend_from_slice(&data);

    // Add padding if necessary
    let mut terminator = 0;
    let terminator_size = match version {
        1..=40 => 4,
        41 => 3,
        42 => 5,
        43 => 7,
        44 => 9,
        _ => panic!("Invalid version"),
    };

    println!("Data len: {}", combined_data.len());

    while combined_data.len() < data_codewords as usize && terminator < terminator_size {
        combined_data.push(false);
        terminator += 1;
    }

    println!("Data codewords len: {}", combined_data.len());

    // Add remainder bits
    if version <= 40 {
        // Add remainder bits so that the length is a multiple of multiple
        while combined_data.len() % 8 != 0 {
            combined_data.push(false);
        }
        let remainding_bytes = (data_codewords as usize - combined_data.len()) / 8;

        for i in 0..remainding_bytes {
            combined_data.extend_from_slice(&REMAINDER_BITS[i % 2]);
        }
    } else if version <= 44 {
        while combined_data.len() < data_codewords as usize {
            combined_data.push(false);
        }
    }

    Ok(combined_data)
}

fn build_segment(mode_indicator: Vec<bool>, size: Vec<bool>, data: Vec<bool>) -> Vec<bool> {
    let mut segment = vec![];

    // Add mode indicator
    segment.extend_from_slice(&mode_indicator);

    // Add size
    segment.extend_from_slice(&size);

    // Add data
    segment.extend_from_slice(&data);

    segment
}

fn get_bit_count(version: usize, mode: &Mode) -> u32 {
    match mode {
        Mode::Numeric => match version {
            1..=9 => 10,
            10..=26 => 12,
            27..=40 => 14,
            41 => 3,
            42 => 4,
            43 => 5,
            44 => 6,
            _ => panic!("Invalid version"),
        },
        Mode::Alphanumeric => match version {
            1..=9 => 9,
            10..=26 => 11,
            27..=40 => 13,
            42 => 3,
            43 => 4,
            44 => 5,
            _ => panic!("Invalid version"),
        },
        Mode::Byte => match version {
            1..=9 => 8,
            10..=26 => 16,
            27..=40 => 16,
            43 => 4,
            44 => 5,
            _ => panic!("Invalid version"),
        },
        Mode::Kanji => match version {
            1..=9 => 8,
            10..=26 => 10,
            27..=40 => 12,
            43 => 3,
            44 => 4,
            _ => panic!("Invalid version"),
        },
        Mode::ECI(mode) => match mode {
            0..=127 => 8,
            128..=16383 => 16,
            16384..=999999 => 24,
            _ => 0,
        },
    }
}

fn get_mode(mode: &Mode, version: usize) -> Vec<bool> {
    match version {
        1..=40 => match mode {
            Mode::Numeric => vec![false, false, false, true],
            Mode::Alphanumeric => vec![false, false, true, false],
            Mode::Byte => vec![false, true, false, false],
            Mode::Kanji => vec![true, false, false, false],
            Mode::ECI(_) => vec![false, true, true, true],
        },
        41 => vec![],
        42 => match mode {
            Mode::Numeric => vec![false],
            Mode::Alphanumeric => vec![true],
            _ => panic!("Invalid mode"),
        },
        43 => match mode {
            Mode::Numeric => vec![false, false],
            Mode::Alphanumeric => vec![false, true],
            Mode::Byte => vec![true, false],
            Mode::Kanji => vec![true, true],
            _ => panic!("Invalid mode"),
        },
        44 => match mode {
            Mode::Numeric => vec![false, false, false],
            Mode::Alphanumeric => vec![false, false, true],
            Mode::Byte => vec![false, true, false],
            Mode::Kanji => vec![false, true, true],
            _ => panic!("Invalid mode"),
        },
        _ => panic!("Invalid version"),
    }
}

fn get_size(bytes: &[u8], bit_count: u32, mode: &Mode) -> Vec<bool> {
    match mode {
        Mode::Kanji => {
            let size = bytes.len() as u32 / 2;
            let mut size_bits = vec![];
            for i in 0..bit_count {
                size_bits.push((size >> (bit_count - i - 1)) & 1 == 1);
            }
            size_bits
        }
        Mode::ECI(mode) => {
            let mut size_bits = vec![];
            for i in 0..bit_count {
                size_bits.push((mode >> (bit_count - i - 1)) & 1 == 1);
            }
            size_bits
        }
        _ => {
            let size = bytes.len() as u32;
            let mut size_bits = vec![];
            for i in 0..bit_count {
                size_bits.push((size >> (bit_count - i - 1)) & 1 == 1);
            }
            size_bits
        }
    }
}

fn get_data(bytes: &[u8], mode: &Mode) -> Vec<bool> {
    let mut data = vec![];

    match mode {
        Mode::Numeric => {
            // Split text into groups of 3
            for chunk in bytes.chunks(3) {
                let mut value = 0;
                for (i, c) in chunk.iter().enumerate() {
                    value += (*c as u32 - '0' as u32) * 10u32.pow((chunk.len() - i - 1) as u32);
                }
                match chunk.len() {
                    1 => {
                        for i in (0..4).rev() {
                            data.push((value >> i) & 1 == 1);
                        }
                    }
                    2 => {
                        for i in (0..7).rev() {
                            data.push((value >> i) & 1 == 1);
                        }
                    }
                    3 => {
                        for i in (0..10).rev() {
                            data.push((value >> i) & 1 == 1);
                        }
                    }
                    _ => {}
                }
            }
        }
        Mode::Alphanumeric => {
            let mut chars = bytes.iter().map(|&c| c as char);
            while let Some(c1) = chars.next() {
                if chars.clone().count() == 0 {
                    let value = get_alphanumeric_index(c1);
                    for i in (0..6).rev() {
                        data.push((value >> i) & 1 == 1);
                    }
                } else {
                    let value = get_alphanumeric_index(c1) * 45
                        + get_alphanumeric_index(chars.next().unwrap());
                    for i in (0..11).rev() {
                        data.push((value >> i) & 1 == 1);
                    }
                }
            }
        }
        Mode::Byte => {
            for byte in bytes {
                for i in (0..8).rev() {
                    data.push((byte >> i) & 1 == 1);
                }
            }
        }
        Mode::Kanji => {
            // Process Shift-JIS bytes in pairs
            for chunk in bytes.chunks(2) {
                if chunk.len() == 2 {
                    // Convert two bytes to 13-bit value
                    let mut value = ((chunk[0] as u16) << 8) | chunk[1] as u16;

                    // Apply Shift-JIS conversion
                    if (0x8140..=0x9FFC).contains(&value) {
                        value -= 0x8140;
                    } else if (0xE040..=0xEBBF).contains(&value) {
                        value -= 0xC140;
                    }

                    // Convert to 13-bit format
                    value = ((value >> 8) * 0xC0) + (value & 0xFF);

                    // Add 13 bits to data
                    for i in (0..13).rev() {
                        data.push((value >> i) & 1 == 1);
                    }
                }
            }
        }
        Mode::ECI(_) => {}
    }

    data
}

fn get_alphanumeric_index(c: char) -> u32 {
    match c {
        '0'..='9' => c as u32 - '0' as u32,
        'A'..='Z' => c as u32 - 'A' as u32 + 10,
        ' ' => 36,
        '$' => 37,
        '%' => 38,
        '*' => 39,
        '+' => 40,
        '-' => 41,
        '.' => 42,
        '/' => 43,
        ':' => 44,
        _ => 0,
    }
}

fn lookup_data_codewords(version: usize, error_correction: &ErrorCorrection) -> u32 {
    // Error correction index mapping

    let ec_index = match error_correction {
        ErrorCorrection::L => 0,
        ErrorCorrection::M => 1,
        ErrorCorrection::Q => 2,
        ErrorCorrection::H => 3,
    };

    // Validate version range and return corresponding value
    if version >= 1 && version <= 44 {
        DATA_CODEWORDS[version - 1][ec_index]
    } else {
        panic!("Invalid version");
    }
}
