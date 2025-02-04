use crate::{error::QRError, ErrorCorrection, Mode};

pub(crate) fn encode_segment (
    version: usize,
    mode: &Mode,
    bytes: &[u8],
) -> Vec<bool> {
    let bit_count = get_bit_count(version, mode);
    let mode_indicator = get_mode(mode);
    let size = get_size(bytes, bit_count, mode);
    let data = get_data(bytes, mode);
    build_segment(mode_indicator, size, data)
}

// remaining bits 11101100 00010001
const REMAINDER_BITS: [[bool; 8]; 2] = [
    [true, true, true, false, true, true, false, false],
    [false, false, false, true, false, false, false, true],
];

pub(crate) fn build_combined_data(
    data: Vec<bool>,
    version: usize,
    error_correction: &ErrorCorrection,
) -> Result<Vec<bool>, QRError> {
    let mut combined_data = vec![];
    
    let data_codewords = lookup_data_codewords(version, error_correction) * 8;

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

    while combined_data.len() + terminator < data_codewords as usize && terminator < 4 {
        combined_data.push(false);
        terminator += 1;
    }

    // Add remainder bits so that the length is a multiple of 8
    while combined_data.len() % 8 != 0 {
        combined_data.push(false);
    }

    // Add remainder bits
    let remainding_bytes = (data_codewords as usize - combined_data.len()) / 8;

    for i in 0..remainding_bytes {
        combined_data.extend_from_slice(&REMAINDER_BITS[i % 2]);
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
            _ => 0,
        },
        Mode::Alphanumeric => match version {
            1..=9 => 9,
            10..=26 => 11,
            27..=40 => 13,
            _ => 0,
        },
        Mode::Byte => match version {
            1..=9 => 8,
            10..=26 => 16,
            27..=40 => 16,
            _ => 0,
        },
        Mode::Kanji => match version {
            1..=9 => 8,
            10..=26 => 10,
            27..=40 => 12,
            _ => 0,
        },
        Mode::ECI(mode) => match mode {
            0..=127 => 8,
            128..=16383 => 16,
            16384..=999999 => 24,
            _ => 0,
        },
    }
}

fn get_mode(mode: &Mode) -> Vec<bool> {
    match mode {
        Mode::Numeric => vec![false, false, false, true],
        Mode::Alphanumeric => vec![false, false, true, false],
        Mode::Byte => vec![false, true, false, false],
        Mode::Kanji => vec![true, false, false, false],
        Mode::ECI(_) => vec![false, true, true, true],
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
        Mode::ECI(_) => {
            
        }
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

const DATA_CODEWORDS: [[u32; 4]; 40] = [
    [19, 16, 13, 9],          // Version 1
    [34, 28, 22, 16],         // Version 2
    [55, 44, 34, 26],         // Version 3
    [80, 64, 48, 36],         // Version 4
    [108, 86, 62, 46],        // Version 5
    [136, 108, 76, 60],       // Version 6
    [156, 124, 88, 66],       // Version 7
    [194, 154, 110, 86],      // Version 8
    [232, 182, 132, 100],     // Version 9
    [274, 216, 154, 122],     // Version 10
    [324, 254, 180, 140],     // Version 11
    [370, 290, 206, 158],     // Version 12
    [428, 334, 244, 180],     // Version 13
    [461, 365, 261, 197],     // Version 14
    [523, 415, 295, 223],     // Version 15
    [589, 453, 325, 253],     // Version 16
    [647, 507, 367, 283],     // Version 17
    [721, 563, 397, 313],     // Version 18
    [795, 627, 445, 341],     // Version 19
    [861, 669, 485, 385],     // Version 20
    [932, 714, 512, 406],     // Version 21
    [1006, 782, 568, 442],    // Version 22
    [1094, 860, 614, 464],    // Version 23
    [1174, 914, 664, 514],    // Version 24
    [1276, 1000, 718, 538],   // Version 25
    [1370, 1062, 754, 596],   // Version 26
    [1468, 1128, 808, 628],   // Version 27
    [1531, 1193, 871, 661],   // Version 28
    [1631, 1267, 911, 701],   // Version 29
    [1735, 1373, 985, 745],   // Version 30
    [1843, 1455, 1033, 793],  // Version 31
    [1955, 1541, 1115, 845],  // Version 32
    [2071, 1631, 1171, 901],  // Version 33
    [2191, 1725, 1231, 961],  // Version 34
    [2306, 1812, 1286, 986],  // Version 35
    [2434, 1914, 1354, 1054], // Version 36
    [2566, 1992, 1426, 1096], // Version 37
    [2702, 2102, 1502, 1142], // Version 38
    [2812, 2216, 1582, 1222], // Version 39
    [2956, 2334, 1666, 1276], // Version 40
];

fn lookup_data_codewords(version: usize, error_correction: &ErrorCorrection) -> u32 {
    // Error correction index mapping

    let ec_index = match error_correction {
        ErrorCorrection::L => 0,
        ErrorCorrection::M => 1,
        ErrorCorrection::Q => 2,
        ErrorCorrection::H => 3,
    };

    // Validate version range and return corresponding value
    if version >= 1 && version <= 40 {
        DATA_CODEWORDS[version - 1][ec_index]
    } else {
        0 // Invalid version
    }
}
