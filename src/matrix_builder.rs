use std::collections::HashSet;

pub fn build_qr_matrix(version: u32, error_correction: &str, data: Vec<bool>) -> Vec<Vec<Option<bool>>> {
    let dimension = get_dimension(version);

    let mut matrix: Vec<Vec<Option<bool>>> = vec![vec![None; dimension as usize]; dimension as usize];

    add_finder_patterns(&mut matrix);

    add_seperators(&mut matrix);

    add_alignment_patterns(&mut matrix, version);

    add_timing_patterns(&mut matrix);

    add_dark_module(&mut matrix, version);

    add_reseverd_area(&mut matrix, version);

    let data_coordinates = add_data(&mut matrix, data);

    let mask = apply_mask(&mut matrix, data_coordinates);

    apply_format_version_information(&mut matrix, version, error_correction, mask);



    
    
    matrix
}

fn add_finder_patterns(matrix: &mut Vec<Vec<Option<bool>>>) {
    let finder_pattern = vec![
        vec![true, true, true, true, true, true, true],
        vec![true, false, false, false, false, false, true],
        vec![true, false, true, true, true, false, true],
        vec![true, false, true, true, true, false, true],
        vec![true, false, true, true, true, false, true],
        vec![true, false, false, false, false, false, true],
        vec![true, true, true, true, true, true, true],
    ];

    let dimension = matrix.len();

    for i in 0..7 {
        for j in 0..7 {
            matrix[i][j] = Some(finder_pattern[i][j]);
            matrix[i][dimension - 1 - j] = Some(finder_pattern[i][j]);
            matrix[dimension - 1 - i][j] = Some(finder_pattern[i][j]);
        }
    }
}

fn add_seperators(matrix: &mut Vec<Vec<Option<bool>>>) {
    let dimension = matrix.len();

    for i in 0..8 {
        matrix[i][7] = Some(false);
        matrix[7][i] = Some(false);
        matrix[i][dimension - 8] = Some(false);
        matrix[7][dimension - 1 - i] = Some(false);
        matrix[dimension - 8][i] = Some(false);
        matrix[dimension - 1 - i][7] = Some(false);
    }
}


const ALIGNMENT_PATTERN: [[bool; 5]; 5] = [
    [true, true, true, true, true],
    [true, false, false, false, true],
    [true, false, true, false, true],
    [true, false, false, false, true],
    [true, true, true, true, true],
];

fn add_alignment_patterns(matrix: &mut Vec<Vec<Option<bool>>>, version: u32) {
    let alignment_location = get_alignment_location(version);

    

    for (x, y) in alignment_location { // center

        if matrix[x as usize][y as usize] != None {
            continue;
        }

        for i in 0..5 {
            for j in 0..5 {
                matrix[x as usize - 2 + i][y as usize - 2 + j] = Some(ALIGNMENT_PATTERN[i][j]);
            }
        }
    }

    
}

fn add_timing_patterns(matrix: &mut Vec<Vec<Option<bool>>>) {
    let dimension = matrix.len();

    for i in 8..dimension - 8 {
        matrix[6][i] = Some(i % 2 == 0);
        matrix[i][6] = Some(i % 2 == 0);
    }
}


fn add_dark_module(matrix: &mut Vec<Vec<Option<bool>>>, version: u32) {

    let x = 8;
    let y = 4 * version + 9;
    
    matrix[y as usize][x as usize] = Some(true);
    
}

fn add_reseverd_area(matrix: &mut Vec<Vec<Option<bool>>>, version: u32) {
    add_reserverd_area(matrix);
    if version >= 7 {
        add_reserverd_area_v7_to_v40(matrix);
    }
    
}


fn add_reserverd_area(matrix: &mut Vec<Vec<Option<bool>>>) {
    let dimension = matrix.len();

    // top left down
    for i in 0..9 {
        if matrix[i][8] == None {
            matrix[i][8] = Some(false);
        }
    }

    // top left right
    for i in 0..8 {
        if matrix[8][i] == None {
            matrix[8][i] = Some(false);
        }
    }

    // down left down
    for i in 0..7 {
        if matrix[dimension - 7 + i][8] == None {
            matrix[dimension - 7 + i][8] = Some(false);
        }
    }

    // up right right
    for i in 0..8 {
        if matrix[8][dimension - 8 + i] == None {
            matrix[8][dimension - 8 + i] = Some(false);
        }
    }
}


fn add_reserverd_area_v7_to_v40(matrix: &mut Vec<Vec<Option<bool>>>) {
    let dimension = matrix.len();

    // down left up
    for i in 0..6 {
        for j in 0..3 {
            if matrix[dimension - 11 + j][i] == None {
                matrix[dimension - 11 + j][i] = Some(false);
            }
        }
    }

    // up right left
    for i in 0..6 {
        for j in 0..3 {
            if matrix[i][dimension - 11 + j] == None {
                matrix[i][dimension - 11 + j] = Some(false);
            }
        }
    }
}

fn add_data(matrix: &mut Vec<Vec<Option<bool>>>, data: Vec<bool>) -> HashSet<(i32, i32)> {
    let dimension = matrix.len() as i32;
    let mut visited = HashSet::new();

    // x = dimension - 1, y = dimension - 1
    let mut current: (i32, i32) = (dimension - 1, dimension - 1);
    let mut direction = true; // false = up, true = down
    let mut data_index = 0;

    while data_index < data.len() && current.0 >= 0 {
        if current.0 == 6 {
            current.0 -= 1;
        }

        if matrix[current.1 as usize][current.0 as usize] == None {
            matrix[current.1 as usize][current.0 as usize] = Some(data[data_index]);
            data_index += 1;

            visited.insert((current.0, current.1));
        }

        if matrix[current.1 as usize][current.0 as usize - 1] == None {
            matrix[current.1 as usize][current.0 as usize - 1] = Some(data[data_index]);
            data_index += 1;

            visited.insert((current.0 - 1, current.1));
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




fn apply_mask(matrix: &mut Vec<Vec<Option<bool>>>, data_coordinates: HashSet<(i32, i32)>) -> u32 {
    let mut mask = 0;
    let mut min_penalty = 1 << 30;

    let mut temp = matrix.clone();

    for i in 0..8 {
        let mut new_matrix = matrix.clone();
        apply_mask_pattern(&mut new_matrix, i, &data_coordinates);

        let penalty = calculate_penalty(&new_matrix);

        if penalty < min_penalty {
            min_penalty = penalty;
            mask = i;
            temp = new_matrix;
        }
    }

    for i in 0..matrix.len() {
        for j in 0..matrix.len() {
            matrix[i][j] = temp[i][j];
        }
    }


    mask
}

fn apply_mask_pattern(matrix: &mut Vec<Vec<Option<bool>>>, mask: u32, data_coordinates: &HashSet<(i32, i32)>) {
    let dimension = matrix.len() as i32;

    for i in 0..dimension {
        for j in 0..dimension {
            if data_coordinates.contains(&(i, j)) {
                match mask {
                    0 => {
                        if (i + j) % 2 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    1 => {
                        if i % 2 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    2 => {
                        if j % 3 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    3 => {
                        if (i + j) % 3 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    4 => {
                        if (i / 2 + j / 3) % 2 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    5 => {
                        if (i * j) % 2 + (i * j) % 3 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    6 => {
                        if ((i * j) % 2 + (i * j) % 3) % 2 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    7 => {
                        if (((i + j) % 2) + ((i * j) % 3)) % 2 == 0 {
                            matrix[i as usize][j as usize] = Some(!matrix[i as usize][j as usize].unwrap());
                        }
                    },
                    _ => {},
                }
            }
        }
    }
}

fn calculate_penalty(matrix: &Vec<Vec<Option<bool>>>) -> i32 {
    let dimension = matrix.len() as i32;

    let mut penalty = 0;

    penalty += calculate_penalty_rule_1(matrix, dimension);
    penalty += calculate_penalty_rule_2(matrix, dimension);
    penalty += calculate_penalty_rule_3(matrix, dimension);
    penalty += calculate_penalty_rule_4(matrix, dimension);

    penalty
}

fn calculate_penalty_rule_1(matrix: &Vec<Vec<Option<bool>>>, dimension: i32) -> i32 {
    let mut penalty = 0;

    // Horizontal
    for i in 0..dimension {
        let mut count = 1;
        let mut current = matrix[0][i as usize];

        for j in 1..dimension {
            if matrix[j as usize][i as usize] == current {
                count += 1;
            } else {
                if count >= 5 {
                    penalty += count - 2;
                }

                count = 1;
                current = matrix[j as usize][i as usize];
            }
        }

        if count >= 5 {
            penalty += count - 2;
        }
    }

    // Vertical
    for i in 0..dimension {
        let mut count = 1;
        let mut current = matrix[i as usize][0];

        for j in 1..dimension {
            if matrix[i as usize][j as usize] == current {
                count += 1;
            } else {
                if count >= 5 {
                    penalty += count - 2;
                }

                count = 1;
                current = matrix[i as usize][j as usize];
            }
        }

        if count >= 5 {
            penalty += count - 2;
        }
    }

    penalty
    
}

fn calculate_penalty_rule_2(matrix: &Vec<Vec<Option<bool>>>, dimension: i32) -> i32 {
    let mut penalty = 0;

    for i in 0..dimension - 1 {
        for j in 0..dimension - 1 {
            if matrix[i as usize][j as usize] == matrix[i as usize][j as usize + 1] &&
               matrix[i as usize][j as usize] == matrix[i as usize + 1][j as usize] &&
               matrix[i as usize][j as usize] == matrix[i as usize + 1][j as usize + 1] {
                penalty += 3;
            }
        }
    }

    penalty
}

const PATTERN: [bool; 11] = [true, false, true, true, true, false, true, false, false, false, false];

fn check_pattern(slice: &[Option<bool>]) -> bool {
    for (i, &val) in slice.iter().enumerate() {
        if val != Some(PATTERN[i]) {
            return false;
        }
    }
    true
}

fn check_reversed_pattern(slice: &[Option<bool>]) -> bool {
    for (i, &val) in slice.iter().enumerate() {
        if val != Some(PATTERN[10 - i]) {
            return false;
        }
    }
    true
}

fn calculate_penalty_rule_3(matrix: &Vec<Vec<Option<bool>>>, dimension: i32) -> i32 {
    let mut penalty = 0;
    let dim = dimension as usize;

    // Horizontal check
    for i in 0..dim {
        for j in 0..(dim - 10) {
            let row_slice = &matrix[i][j..j+11];
            if check_pattern(row_slice) || check_reversed_pattern(row_slice) {
                penalty += 40;
            }
        }
    }

    // Vertical check
    for j in 0..dim {
        for i in 0..(dim - 10) {
            // Collect the 11 vertical items; store them in a small local array
            let mut column_slice = [None; 11];
            for k in 0..11 {
                column_slice[k] = matrix[i + k][j];
            }
            if check_pattern(&column_slice) || check_reversed_pattern(&column_slice) {
                penalty += 40;
            }
        }
    }

    penalty
}
fn calculate_penalty_rule_4(matrix: &Vec<Vec<Option<bool>>>, dimension: i32) -> i32 {
    let mut dark_count = 0;

    for i in 0..dimension {
        for j in 0..dimension {
            if matrix[i as usize][j as usize].unwrap() {
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


fn apply_format_version_information(matrix: &mut Vec<Vec<Option<bool>>>, version: u32, error_correction: &str, mask: u32) {

    let dimension = matrix.len();

    if version >= 7 {
        let version_information = get_version_information(version);
        let mut version_information_index = 0;

        for i in 0..6 {
            for j in 0..3 {
                matrix[5 - i as usize][dimension- 11 + 2 - j as usize] = Some(version_information[version_information_index]);
                matrix[dimension - 11 + 2 - j as usize][5 - i as usize] = Some(version_information[version_information_index]);

                version_information_index += 1;
            }
        }

        
    }
    let format_information_string = get_format_information(error_correction, mask);

    // top left
    let mut format_information_index = 0;

    for i in 0..9 {
        if i != 6 {
            matrix[8][i] = Some(format_information_string[format_information_index]);
            format_information_index += 1;
        }
    }

    for i in 0..8 {
        if (7 - i) != 6 {
            matrix[7-i][8] = Some(format_information_string[format_information_index]);
            format_information_index += 1;
        }
    }

    // right bottom
    format_information_index = 0;

    for i in 0..7 {
        matrix[dimension - 1 - i][8] = Some(format_information_string[format_information_index]);
        format_information_index += 1;
    }

    for i in 0..8 {
        matrix[8][dimension - 8 + i] = Some(format_information_string[format_information_index]);
        format_information_index += 1;
    }



    
}


fn get_dimension(version: u32) -> u32 {
    (version - 1) * 4 + 21
}

fn get_alignment_location(version: u32) -> Vec<(u32, u32)> {
    let mut alignment_pattern = Vec::new();

    if version == 1 {
        return alignment_pattern;
    }

    for i in 0..ALIGNMENT_PATTERN_LOCATION[version as usize - 2].len() {
        for j in 0..ALIGNMENT_PATTERN_LOCATION[version as usize - 2].len() {
            alignment_pattern.push((ALIGNMENT_PATTERN_LOCATION[version as usize - 2][i], ALIGNMENT_PATTERN_LOCATION[version as usize - 2][j]));
        }
    }

    alignment_pattern
}

const ALIGNMENT_PATTERN_LOCATION: [&[u32]; 39] = [
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

fn get_format_information(error_correction: &str, mask: u32) -> Vec<bool> {
    let ec_level = match error_correction {
        "L" => 0,
        "M" => 1,
        "Q" => 2,
        "H" => 3,
        _ => panic!("Invalid error correction level"),
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

fn get_version_information(version: u32) -> Vec<bool> {
    let version_info = VERSION_INFORMATION[version as usize - 7];
    
    let mut version_information = Vec::new();

    for c in version_info.chars() {
        version_information.push(c == '1');
    }

    version_information
}



