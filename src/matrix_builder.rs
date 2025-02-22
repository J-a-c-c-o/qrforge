use crate::{
    constants::{
        ALIGNMENT_PATTERN, ALIGNMENT_PATTERN_LOCATION, FINDER_PATTERN, FORMAT_INFORMATION, PATTERN,
        VERSION_INFORMATION,
    },
    qrcode::{self, QRCode},
    ErrorCorrection,
};
#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Build the QR matrix
pub(crate) fn build_qr_matrix(
    matrix: &mut qrcode::QRCode,
    version: usize,
    error_correction: &ErrorCorrection,
    data: Vec<bool>,
) {
    add_finder_patterns(matrix);

    add_separators(matrix);

    add_alignment_patterns(matrix, version);

    add_timing_patterns(matrix);

    add_dark_module(matrix, version);

    add_reseverd_area(matrix, version);

    let data_coordinates = add_data(matrix, data);

    let mask = apply_mask(matrix, data_coordinates);
    apply_format_version_information(matrix, version, error_correction, mask);
}

/// Add the finder patterns
fn add_finder_patterns(matrix: &mut qrcode::QRCode) {
    let dimension = matrix.dimension();

    FINDER_PATTERN.iter().enumerate().for_each(|(i, row)| {
        row.iter().enumerate().for_each(|(j, &value)| {
            matrix.set(j, i, value);
            matrix.set(dimension - 1 - j, i, value);
            matrix.set(j, dimension - 1 - i, value);
        });
    });
}

/// Add the separators
fn add_separators(matrix: &mut qrcode::QRCode) {
    let dimension = matrix.dimension();

    for i in 0..8 {
        matrix.set(7, i, false);
        matrix.set(i, 7, false);
        matrix.set(dimension - 8, i, false);
        matrix.set(dimension - 1 - i, 7, false);
        matrix.set(i, dimension - 8, false);
        matrix.set(7, dimension - 1 - i, false);
    }
}

/// Add the alignment patterns
fn add_alignment_patterns(matrix: &mut QRCode, version: usize) {
    let alignment_location = get_alignment_location(version);

    for (x, y) in alignment_location {
        // center

        if matrix.get(x, y) {
            continue;
        }

        ALIGNMENT_PATTERN.iter().enumerate().for_each(|(i, row)| {
            row.iter().enumerate().for_each(|(j, &value)| {
                matrix.set(x - 2 + i, y - 2 + j, value);
            });
        });
    }
}

/// Add the timing patterns
fn add_timing_patterns(matrix: &mut QRCode) {
    let dimension = matrix.dimension();

    for i in 8..dimension - 8 {
        matrix.set(i, 6, i % 2 == 0);
        matrix.set(6, i, i % 2 == 0);
    }
}

/// Add the dark module
fn add_dark_module(matrix: &mut QRCode, version: usize) {
    let x = 8;
    let y = 4 * version + 9;

    matrix.set(x, y, true);
}

/// Add the reserved area
fn add_reseverd_area(matrix: &mut QRCode, version: usize) {
    add_reserverd_area(matrix);
    if version >= 7 {
        add_reserverd_area_v7_to_v40(matrix);
    }
}

/// Add the reserved area
fn add_reserverd_area(matrix: &mut QRCode) {
    let dimension = matrix.dimension();

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

/// Add the reserved area for version 7 to 40
fn add_reserverd_area_v7_to_v40(matrix: &mut QRCode) {
    let dimension = matrix.dimension();

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

/// Add the data to the matrix
fn add_data(matrix: &mut QRCode, data: Vec<bool>) -> Vec<(i32, i32)> {
    let dimension = matrix.dimension() as i32;
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

/// Apply the mask
fn apply_mask(matrix: &mut QRCode, data_coordinates: Vec<(i32, i32)>) -> u32 {
    {
        #[cfg(not(feature = "parallel"))]
        let results: Vec<(u32, i32, QRCode)> = (0..8)
            .map(|i| {
                let mut new_matrix = matrix.clone();
                apply_mask_pattern(&mut new_matrix, i, &data_coordinates);
                let penalty = calculate_penalty(&new_matrix);
                (i, penalty, new_matrix)
            })
            .collect();

        #[cfg(not(feature = "parallel"))]
        let (mask, _, best_matrix) = results
            .iter()
            .min_by_key(|(_, penalty, _)| *penalty)
            .unwrap();

        #[cfg(feature = "parallel")]
        let results: Vec<(u32, i32, QRCode)> = (0..8)
            .into_par_iter()
            .map(|i| {
                let mut new_matrix = matrix.clone();
                apply_mask_pattern(&mut new_matrix, i, &data_coordinates);
                let penalty = calculate_penalty(&new_matrix);
                (i, penalty, new_matrix)
            })
            .collect();

        #[cfg(feature = "parallel")]
        let (mask, _, best_matrix) = results
            .par_iter()
            .min_by_key(|(_, penalty, _)| *penalty)
            .unwrap();

        for i in 0..matrix.dimension() {
            for j in 0..matrix.dimension() {
                matrix.set(j, i, best_matrix.get(j, i));
            }
        }

        *mask

    }
}

/// Apply the mask pattern
fn apply_mask_pattern(matrix: &mut QRCode, mask: u32, data_coordinates: &[(i32, i32)]) {
    for (x, y) in data_coordinates.iter() {
        let i = *y as usize;
        let j = *x as usize;
        match mask {
            0 => {
                if (i + j) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            1 => {
                if i % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            2 => {
                if j % 3 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            3 => {
                if (i + j) % 3 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            4 => {
                if (i / 2 + j / 3) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            5 => {
                if (i * j) % 2 + (i * j) % 3 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            6 => {
                if ((i * j) % 2 + (i * j) % 3) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            7 => {
                if (((i + j) % 2) + ((i * j) % 3)) % 2 == 0 {
                    matrix.set(j, i, !matrix.get(j, i));
                }
            }
            _ => {}
        }
    }
}

/// Calculate the penalty
fn calculate_penalty(matrix: &QRCode) -> i32 {
    let mut penalty = 0;

    penalty += calculate_penalty_rule_1(matrix);
    penalty += calculate_penalty_rule_2(matrix);
    penalty += calculate_penalty_rule_3(matrix);
    penalty += calculate_penalty_rule_4(matrix);

    penalty
}

/// Calculate the penalty for rule 1
fn calculate_penalty_rule_1(matrix: &QRCode) -> i32 {
    let mut penalty = 0;

    let dimension = matrix.dimension();

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

/// Calculate the penalty for rule 2
fn calculate_penalty_rule_2(matrix: &QRCode) -> i32 {
    let boxes = count_boxes(matrix);

    boxes * 3
}

/// Count the number of boxes
fn count_boxes(matrix: &QRCode) -> i32 {
    let dimension = matrix.dimension();
    let mut count = 0;

    for i in 0..dimension - 1 {
        for j in 0..dimension - 1 {
            if matrix.get(j, i) == matrix.get(j + 1, i)
                && matrix.get(j, i) == matrix.get(j, i + 1)
                && matrix.get(j, i) == matrix.get(j + 1, i + 1)
            {
                count += 1;
            }
        }
    }

    count
}

/// Calculate the penalty for rule 3
fn calculate_penalty_rule_3(matrix: &QRCode) -> i32 {
    count_occurences(matrix, &PATTERN) * 40
}

fn count_occurences(matrix: &QRCode, pattern: &[bool; 7]) -> i32 {
    // maybe use bit manipulation
    let mut count = 0;
    let dimension = matrix.dimension();

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

            if current_count == 7 {
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

            if current_count == 7 {
                count += 1;
                current = 0;
                current_count = 0;
            }
        }
    }

    count
}

/// Calculate the penalty for rule 4
fn calculate_penalty_rule_4(matrix: &QRCode) -> i32 {
    let mut dark_count = 0;
    let dimension = matrix.dimension();

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

/// Apply the format and version information
fn apply_format_version_information(
    matrix: &mut QRCode,
    version: usize,
    error_correction: &ErrorCorrection,
    mask: u32,
) {
    let dimension = matrix.dimension();

    if version >= 7 {
        let version_information = get_version_information(version);
        let mut version_information_index = 0;

        for i in 0..6 {
            for j in 0..3 {
                matrix.set(
                    dimension - 11 + 2 - j,
                    5 - i,
                    version_information[version_information_index],
                );
                matrix.set(
                    5 - i,
                    dimension - 11 + 2 - j,
                    version_information[version_information_index],
                );

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
            matrix.set(
                8,
                7 - i,
                format_information_string[format_information_index],
            );
            format_information_index += 1;
        }
    }

    // right bottom
    format_information_index = 0;

    for i in 0..7 {
        matrix.set(
            8,
            dimension - 1 - i,
            format_information_string[format_information_index],
        );
        format_information_index += 1;
    }

    for i in 0..8 {
        matrix.set(
            dimension - 8 + i,
            8,
            format_information_string[format_information_index],
        );
        format_information_index += 1;
    }
}

/// Get the alignment location
fn get_alignment_location(version: usize) -> Vec<(usize, usize)> {
    let mut alignment_pattern = Vec::new();

    if version == 1 {
        return alignment_pattern;
    }

    for i in 0..ALIGNMENT_PATTERN_LOCATION[version - 2].len() {
        for j in 0..ALIGNMENT_PATTERN_LOCATION[version - 2].len() {
            alignment_pattern.push((
                ALIGNMENT_PATTERN_LOCATION[version - 2][i],
                ALIGNMENT_PATTERN_LOCATION[version - 2][j],
            ));
        }
    }

    alignment_pattern
}

/// Get the format information
fn get_format_information(error_correction: &ErrorCorrection, mask: u32) -> Vec<bool> {
    let ec_level = match error_correction {
        ErrorCorrection::L => 1,
        ErrorCorrection::M => 0,
        ErrorCorrection::Q => 3,
        ErrorCorrection::H => 2,
    };

    let index = (ec_level << 3) | mask;

    let format_info = FORMAT_INFORMATION[index as usize];

    let mut format_information = Vec::new();

    for i in (0..15).rev() {
        format_information.push((format_info >> i) & 1 == 1);
    }

    format_information
}

/// Get the version information
fn get_version_information(version: usize) -> Vec<bool> {
    let version_info = VERSION_INFORMATION[version - 7];

    let mut version_information = Vec::new();

    for i in (0..18).rev() {
        version_information.push((version_info >> i) & 1 == 1);
    }

    version_information
}
