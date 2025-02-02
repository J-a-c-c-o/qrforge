use rayon::prelude::*;
use crate::{ErrorCorrection, QRCode};




pub fn build_qr_matrix(matrix: &mut QRCode, version: usize, error_correction: &ErrorCorrection, data: Vec<bool>) {


    add_finder_patterns(matrix);

    add_seperators(matrix);

    add_alignment_patterns(matrix, version);

    add_timing_patterns(matrix);

    add_dark_module(matrix, version as usize);

    add_reseverd_area(matrix, version as usize);

    let data_coordinates = add_data(matrix, data);

    let mask = apply_mask(matrix, data_coordinates);
    apply_format_version_information(matrix, version, error_correction, mask);
}


const FINDER_PATTERN: [[bool; 7]; 7] = [
    [true, true, true, true, true, true, true],
    [true, false, false, false, false, false, true],
    [true, false, true, true, true, false, true],
    [true, false, true, true, true, false, true],
    [true, false, true, true, true, false, true],
    [true, false, false, false, false, false, true],
    [true, true, true, true, true, true, true],
];

fn add_finder_patterns(matrix: &mut QRCode) {
    

    let dimension = matrix.len();

    for i in 0..7 {
        for j in 0..7 {
            matrix.set(j, i, FINDER_PATTERN[i][j]);
            matrix.set(dimension - 1 - j, i, FINDER_PATTERN[i][j]);
            matrix.set(j, dimension - 1 - i, FINDER_PATTERN[i][j]);
        }
    }
}

fn add_seperators(matrix: &mut QRCode) {
    let dimension = matrix.len();

    for i in 0..8 {
        matrix.set(7, i, false);
        matrix.set(i, 7, false);
        matrix.set(dimension - 8, i, false);
        matrix.set(dimension - 1 - i, 7, false);
        matrix.set(i, dimension - 8, false);
        matrix.set(7, dimension - 1 - i, false);
    }
}


const ALIGNMENT_PATTERN: [[bool; 5]; 5] = [
    [true, true, true, true, true],
    [true, false, false, false, true],
    [true, false, true, false, true],
    [true, false, false, false, true],
    [true, true, true, true, true],
];

fn add_alignment_patterns(matrix: &mut QRCode, version: usize) {
    let alignment_location = get_alignment_location(version);

    

    for (x, y) in alignment_location { // center

        if matrix.get(x, y) {
            continue;
        }

        for i in 0..5 {
            for j in 0..5 {
                matrix.set(x - 2 + i, y - 2 + j, ALIGNMENT_PATTERN[j][i]);
            }
        }
    }

    
}

fn add_timing_patterns(matrix: &mut QRCode) {
    let dimension = matrix.len();

    for i in 8..dimension - 8 {
        matrix.set(i, 6, i % 2 == 0);
        matrix.set(6, i, i % 2 == 0);
    }
}


fn add_dark_module(matrix: &mut QRCode, version: usize) {

    let x = 8;
    let y = 4 * version + 9;
    
    matrix.set(x, y, true);
    
}

fn add_reseverd_area(matrix: &mut QRCode, version: usize) {
    add_reserverd_area(matrix);
    if version >= 7 {
        add_reserverd_area_v7_to_v40(matrix);
    }
    
}


fn add_reserverd_area(matrix: &mut QRCode) {
    let dimension = matrix.len();

    // top left down
    for i in 0..9 {

        if matrix.is_empty(8, i) {
            matrix.set(8, i, false);
        }
    }

    // top left right
    for i in 0..8 {

        if matrix.is_empty(i, 8) {
            matrix.set(i, 8, false);
        }
    }

    // down left down
    for i in 0..7 {

        if matrix.is_empty(8, dimension - 7 + i) {
            matrix.set(8, dimension - 7 + i, false);
        }
    }

    // up right right
    for i in 0..8 {

        if matrix.is_empty(dimension - 8 + i, 8) {
            matrix.set(dimension - 8 + i, 8, false);
        }
    }
}


fn add_reserverd_area_v7_to_v40(matrix: &mut QRCode) {
    let dimension = matrix.len();

    // down left up
    for i in 0..6 {
        for j in 0..3 {
            if matrix.is_empty(i, dimension - 11 + j) {
                matrix.set(i, dimension - 11 + j, false);
            }
        }
    }

    // up right left
    for i in 0..6 {
        for j in 0..3 {
            if matrix.is_empty(dimension - 11 + j, i) {
                matrix.set(dimension - 11 + j, i, false);
            }
        }
    }
}

fn add_data(matrix: &mut QRCode, data: Vec<bool>) -> Vec<(i32, i32)> {
    let dimension = matrix.len() as i32;
    let mut visited = Vec::new();

    let mut current: (i32, i32) = (dimension - 1, dimension - 1);
    let mut direction = true; // false = up, true = down
    let mut data_index = 0;

    while data_index < data.len() && current.0 >= 0 {
        if current.0 == 6 {
            current.0 -= 1;
        }

        if matrix.is_empty(current.0 as usize, current.1 as usize) {
            matrix.set(current.0 as usize, current.1 as usize, data[data_index]);
            data_index += 1;

            visited.push((current.0, current.1));
        }

        if matrix.is_empty(current.0 as usize - 1, current.1 as usize) {
            matrix.set(current.0 as usize - 1, current.1 as usize, data[data_index]);
            data_index += 1;

            visited.push((current.0 - 1, current.1));
        }

        if direction {
            current.1 -= 1;
        } else {
            current.1 += 1;
        }

        if current.1 == dimension {
            current.1 -= 1;
            current.0 -= 2;
            direction = !direction;
        } else if current.1 == -1 {
            current.1 += 1;
            current.0 -= 2;
            direction = !direction;
        }
        
    }

    visited
}



fn apply_mask(matrix: &mut QRCode, data_coordinates: Vec<(i32, i32)>) -> u32 {
    {
        let results: Vec<(u32, i32, QRCode)> = (0..8)
            .into_par_iter()
            .map(|i| {
                let mut new_matrix = matrix.clone();
                apply_mask_pattern(&mut new_matrix, i, &data_coordinates);
                let penalty = calculate_penalty(&new_matrix);
                (i, penalty, new_matrix)
            })
            .collect();

        let (_, _, best_matrix) = results
            .par_iter()
            .min_by_key(|(_, penalty, _)| *penalty)
            .unwrap();

        for i in 0..matrix.len() {
            for j in 0..matrix.len() {
                matrix.set(j, i, best_matrix.get(j, i));
            }
        }

        results.iter().min_by_key(|(_, penalty, _)| *penalty).unwrap().0
    }
}

fn apply_mask_pattern(matrix: &mut QRCode, mask: u32, data_coordinates: &Vec<(i32, i32)>) {
    for (x, y) in data_coordinates.iter() {
        let i = *y as usize;
        let j = *x as usize;
        match mask {
            0 => {
                if (i + j) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            1 => {
                if i % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            2 => {
                if j % 3 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            3 => {
                if (i + j) % 3 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            4 => {
                if (i / 2 + j / 3) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            5 => {
                if (i * j) % 2 + (i * j) % 3 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            6 => {
                if ((i * j) % 2 + (i * j) % 3) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            7 => {
                if (((i + j) % 2) + ((i * j) % 3)) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            },
            _ => {},
        }
    }
}

fn calculate_penalty(matrix: &QRCode) -> i32 {
    let mut penalty = 0;

    penalty += calculate_penalty_rule_1(matrix);
    penalty += calculate_penalty_rule_2(matrix);
    penalty += calculate_penalty_rule_3(matrix);
    penalty += calculate_penalty_rule_4(matrix);

    penalty
}

fn calculate_penalty_rule_1(matrix: &QRCode) -> i32 {
    let mut penalty = 0;

    let dimension = matrix.len();

    // Horizontal
    for i in 0..dimension {
        let mut count = 1;
        let mut current = matrix.get(i, 0);

        for j in 1..dimension {
            if matrix.get(i, j) == current {
                count += 1;
            } else {
                if count >= 5 {
                    penalty += count - 2;
                }

                count = 1;
                current = matrix.get(i, j);
            }
        }

        if count >= 5 {
            penalty += count - 2;
        }
    }

    // Vertical
    for i in 0..dimension {
        let mut count = 1;
        let mut current = matrix.get(0, i);

        for j in 1..dimension {
            if matrix.get(j, i) == current {
                count += 1;
            } else {
                if count >= 5 {
                    penalty += count - 2;
                }

                count = 1;
                current = matrix.get(j, i);
            }
        }

        if count >= 5 {
            penalty += count - 2;
        }
    }

    penalty
    
}

fn calculate_penalty_rule_2(matrix: &QRCode) -> i32 {
    let boxes = count_boxes(matrix);

    let penalty = boxes*3;
    penalty
}

fn count_boxes(matrix: &QRCode) -> i32 {
    let dimension = matrix.len();
    let mut count = 0;

    for i in 0..dimension - 1 {
        for j in 0..dimension - 1 {
            if matrix.get(j, i) == matrix.get(j + 1, i) && matrix.get(j, i) == matrix.get(j, i + 1) && matrix.get(j, i) == matrix.get(j + 1, i + 1) {
                count += 1;
            }
        }
    }

    count
    
}

const PATTERN: [bool; 11] = [true, false, true, true, true, false, true, false, false, false, false];

const REVERSED_PATTERN: [bool; 11] = [false, false, false, false, true, false, true, true, true, false, true];

fn calculate_penalty_rule_3(matrix: &QRCode) -> i32 {
    let penalty = count_occurences(matrix, &PATTERN) * 40 + count_occurences(matrix, &REVERSED_PATTERN) * 40;

    penalty
    
}


fn count_occurences(matrix: &QRCode, pattern: &[bool; 11]) -> i32 {
    // maybe use bit manipulation
    let mut count = 0;
    let dimension = matrix.len();

    // Horizontal use && to check if it contains the pattern
    for i in 0..dimension {
        let mut current = 0;
        let mut current_count = 0;

        for j in 0..dimension {
            if matrix.get(j, i) == pattern[current] {
                current_count += 1;
                current += 1;
            } else {
                current = 0;
                current_count = 0;
            }

            if current_count == 11 {
                count += 1;
                current = 0;
                current_count = 0;
            }
        }
    }

    // Vertical
    for i in 0..dimension {
        let mut current = 0;
        let mut current_count = 0;

        for j in 0..dimension {
            if matrix.get(i, j) == pattern[current] {
                current_count += 1;
                current += 1;
            } else {
                current = 0;
                current_count = 0;
            }

            if current_count == 11 {
                count += 1;
                current = 0;
                current_count = 0;
            }
        }
    }

    count
    
    
}




fn calculate_penalty_rule_4(matrix: &QRCode) -> i32 {
    let mut dark_count = 0;
    let dimension = matrix.len();

    for i in 0..dimension {
        for j in 0..dimension {
            if matrix.get(j, i) {
                dark_count += 1;
            }
        }
    }

    let percentage = dark_count as f32 / (dimension * dimension) as f32 * 100.0;

    let temp = percentage / 5.0;
    let percentage_upper = temp.ceil() as i32;
    let percentage_lower = temp.floor() as i32;

    let penalty_upper = (percentage_upper - 10).abs();
    let penalty_lower = (percentage_lower - 10).abs();

    if penalty_upper < penalty_lower {
        penalty_upper * 10
    } else {
        penalty_lower * 10
    }

    
}


fn apply_format_version_information(matrix: &mut QRCode, version: usize, error_correction: &ErrorCorrection, mask: u32) {

    let dimension = matrix.len();

    if version >= 7 {
        let version_information = get_version_information(version);
        let mut version_information_index = 0;

        for i in 0..6 {
            for j in 0..3 {
                matrix.set(dimension - 11 + 2 - j, 5 - i, version_information[version_information_index]);
                matrix.set(5 - i, dimension - 11 + 2 - j, version_information[version_information_index]);

                version_information_index += 1;
            }
        }

        
    }
    let format_information_string = get_format_information(error_correction, mask);

    // top left
    let mut format_information_index = 0;

    for i in 0..9 {
        if i != 6 {
            matrix.set(i, 8, format_information_string[format_information_index]);
            format_information_index += 1;
        }
    }

    for i in 0..8 {
        if (7 - i) != 6 {
            matrix.set(8, 7 - i, format_information_string[format_information_index]);
            format_information_index += 1;
        }
    }

    // right bottom
    format_information_index = 0;

    for i in 0..7 {
        matrix.set(8, dimension - 1 - i, format_information_string[format_information_index]);
        format_information_index += 1;
    }

    for i in 0..8 {
        matrix.set(dimension - 8 + i, 8, format_information_string[format_information_index]);
        format_information_index += 1;
    }



    
}




fn get_alignment_location(version: usize) -> Vec<(usize, usize)> {
    let mut alignment_pattern = Vec::new();

    if version == 1 {
        return alignment_pattern;
    }

    for i in 0..ALIGNMENT_PATTERN_LOCATION[version - 2].len() {
        for j in 0..ALIGNMENT_PATTERN_LOCATION[version - 2].len() {
            alignment_pattern.push((ALIGNMENT_PATTERN_LOCATION[version - 2][i], ALIGNMENT_PATTERN_LOCATION[version - 2][j]));
        }
    }

    alignment_pattern
}

const ALIGNMENT_PATTERN_LOCATION: [&[usize]; 39] = [
    &[6, 18], &[6, 22], &[6, 26], &[6, 30], &[6, 34], &[6, 22, 38], &[6, 24, 42], &[6, 26, 46], &[6, 28, 50], &[6, 30, 54],
    &[6, 32, 58], &[6, 34, 62], &[6, 26, 46, 66], &[6, 26, 48, 70], &[6, 26, 50, 74], &[6, 30, 54, 78], &[6, 30, 56, 82], &[6, 30, 58, 86], &[6, 34, 62, 90], &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98], &[6, 30, 54, 78, 102], &[6, 28, 54, 80, 106], &[6, 32, 58, 84, 110], &[6, 30, 58, 86, 114], &[6, 34, 62, 90, 118], &[6, 26, 50, 74, 98, 122], &[6, 30, 54, 78, 102, 126], &[6, 26, 52, 78, 104, 130], &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138], &[6, 30, 58, 86, 114, 142], &[6, 34, 62, 90, 118, 146], &[6, 30, 54, 78, 102, 126, 150], &[6, 24, 50, 76, 102, 128, 154], &[6, 28, 54, 80, 106, 132, 158], &[6, 32, 58, 84, 110, 136, 162], &[6, 26, 54, 82, 110, 138, 166], &[6, 30, 58, 86, 114, 142, 170]
];

const FORMAT_INFORMATION: [[&str; 8]; 4] = [
    ["111011111000100", "111001011110011", "111110110101010", "111100010011101", "110011000101111", "110001100011000", "110110001000001", "110100101110110"],
    ["101010000010010", "101000100100101", "101111001111100", "101101101001011", "100010111111001", "100000011001110", "100111110010111", "100101010100000"],
    ["011010101011111", "011000001101000", "011111100110001", "011101000000110", "010010010110100", "010000110000011", "010111011011010", "010101111101101"],
    ["001011010001001", "001001110111110", "001110011100111", "001100111010000", "000011101100010", "000001001010101", "000110100001100", "000100000111011"]
];

fn get_format_information(error_correction: &ErrorCorrection, mask: u32) -> Vec<bool> {
    let ec_level = match error_correction {
        ErrorCorrection::L => 0,
        ErrorCorrection::M => 1,
        ErrorCorrection::Q => 2,
        ErrorCorrection::H => 3,
    };

    let format_info = FORMAT_INFORMATION[ec_level][mask as usize];
    
    let mut format_information = Vec::new();

    for c in format_info.chars() {
        format_information.push(c == '1');
    }

    format_information
}

const VERSION_INFORMATION : [&str; 34] = [
    "000111110010010100", "001000010110111100", "001001101010011001", "001010010011010011", "001011101111110110",
    "001100011101100010", "001101100001000111", "001110011000001101", "001111100100101000", "010000101101111000",
    "010001010001011101", "010010101000010111", "010011010100110010", "010100100110100110", "010101011010000011",
    "010110100011001001", "010111011111101100", "011000111011000100", "011001000111100001", "011010111110101011",
    "011011000010001110", "011100110000011010", "011101001100111111", "011110110101110101", "011111001001010000",
    "100000100111010101", "100001011011110000", "100010100010111010", "100011011110011111", "100100101100001011",
    "100101010000101110", "100110101001100100", "100111010101000001", "101000110001101001"
];

fn get_version_information(version: usize) -> Vec<bool> {
    let version_info = VERSION_INFORMATION[version - 7];
    
    let mut version_information = Vec::new();

    for c in version_info.chars() {
        version_information.push(c == '1');
    }

    version_information
}



