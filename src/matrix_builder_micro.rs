use crate::{ErrorCorrection, QRCode};
use rayon::prelude::*;

pub(crate) fn build_qr_matrix(
    matrix: &mut QRCode,
    version: usize,
    error_correction: &ErrorCorrection,
    data: Vec<bool>,
) {
    add_finder_patterns(matrix);

    add_seperators(matrix);

    add_timing_patterns(matrix);

    add_reseverd_area(matrix);

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
    for i in 0..7 {
        for j in 0..7 {
            matrix.set(j, i, FINDER_PATTERN[i][j]);
        }
    }
}

fn add_seperators(matrix: &mut QRCode) {
    for i in 0..8 {
        matrix.set(7, i, false);
        matrix.set(i, 7, false);
    }
}

fn add_timing_patterns(matrix: &mut QRCode) {
    let dimension = matrix.dimension();

    for i in 8..dimension {
        matrix.set(i, 0, i % 2 == 0);
        matrix.set(0, i, i % 2 == 0);
    }
}

fn add_reseverd_area(matrix: &mut QRCode) {
    add_reserverd_area(matrix);
}

fn add_reserverd_area(matrix: &mut QRCode) {
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
}

fn add_data(matrix: &mut QRCode, data: Vec<bool>) -> Vec<(i32, i32)> {
    let dimension = matrix.dimension() as i32;
    let mut visited = Vec::new();

    let mut current: (i32, i32) = (dimension - 1, dimension - 1);
    let mut direction = true; // false = up, true = down
    let mut data_index = 0;
    while data_index < data.len() && current.0 >= 0 {
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
        let results: Vec<(u32, i32, QRCode)> = (0..4)
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

        for i in 0..matrix.dimension() {
            for j in 0..matrix.dimension() {
                matrix.set(j, i, best_matrix.get(j, i));
            }
        }

        results
            .iter()
            .min_by_key(|(_, penalty, _)| *penalty)
            .unwrap()
            .0
    }
}

fn apply_mask_pattern(matrix: &mut QRCode, mask: u32, data_coordinates: &Vec<(i32, i32)>) {
    for (x, y) in data_coordinates.iter() {
        let i = *y as usize;
        let j = *x as usize;
        match mask {
            0 => {
                if i % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            1 => {
                if (i / 2 + j / 3) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            2 => {
                if ((i * j) % 2 + (i * j) % 3) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            3 => {
                if (((i + j) % 2) + ((i * j) % 3)) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            _ => {}
        }
    }
}

fn calculate_penalty(matrix: &QRCode) -> i32 {
    let dimension = matrix.dimension();

    let mut sum1 = 0;
    let mut sum2 = 0;
    for i in 1..dimension {
        sum1 += matrix.get(i, dimension - 1) as i32;
        sum2 += matrix.get(dimension - 1, i) as i32;
    }

    let penalty = match sum1 <= sum2 {
        true => sum1 * 16 + sum2,
        false => sum2 * 16 + sum1,
    };

    penalty
}

fn apply_format_version_information(
    matrix: &mut QRCode,
    version: usize,
    error_correction: &ErrorCorrection,
    mask: u32,
) {
    let format_information_string = get_format_information(error_correction, version, mask);

    // top left
    let mut format_information_index = 0;

    for i in 1..=8 {
        matrix.set(8, i, format_information_string[format_information_index]);
        format_information_index += 1;
    }

    for i in 8..=1 {
        matrix.set(i, 8, format_information_string[format_information_index]);
        format_information_index += 1;
    }
}

const FORMAT_INFO_MICRO: [u16; 32] = [
    0x4445, 0x4172, 0x4e2b, 0x4b1c, 0x55ae, 0x5099, 0x5fc0, 0x5af7, 0x6793, 0x62a4, 0x6dfd, 0x68ca,
    0x7678, 0x734f, 0x7c16, 0x7921, 0x06de, 0x03e9, 0x0cb0, 0x0987, 0x1735, 0x1202, 0x1d5b, 0x186c,
    0x2508, 0x203f, 0x2f66, 0x2a51, 0x34e3, 0x31d4, 0x3e8d, 0x3bba,
];

const MICRO_MAPPING: [[u32; 4]; 4] = [[0, 0, 0, 0], [1, 2, 0, 0], [3, 4, 0, 0], [5, 6, 7, 0]];

fn get_format_information(
    error_correction: &ErrorCorrection,
    version: usize,
    mask: u32,
) -> Vec<bool> {
    let ec_level = error_correction.to_value();

    let version = version - 41;

    let mapping = MICRO_MAPPING[version][ec_level] << 2 | mask;

    let format_information = FORMAT_INFO_MICRO[mapping as usize];

    let mut format_information_string = Vec::new();

    for i in 0..15 {
        format_information_string.push((format_information >> i) & 1 == 1);
    }

    format_information_string
}
