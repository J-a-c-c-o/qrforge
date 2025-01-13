pub fn build_QR_matrix(version: u32, data: Vec<bool>) -> Vec<Vec<Option<bool>>> {
    let dimension = get_dimension(version);

    let mut matrix: Vec<Vec<Option<bool>>> = vec![vec![None; dimension as usize]; dimension as usize];

    add_finder_patterns(&mut matrix);

    add_seperators(&mut matrix);

    add_alignment_patterns(&mut matrix, version);

    add_timing_patterns(&mut matrix);

    add_dark_module(&mut matrix, version);

    // pretty print the matrix
    for i in 0..dimension as usize {
        for j in 0..dimension as usize {
            if matrix[i][j] == None {
                print!(" ");
            } else {
                print!("{}", if matrix[i][j].unwrap() { "X" } else { " " });
            }
        }
        println!();
    }

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

fn add_alignment_patterns(matrix: &mut Vec<Vec<Option<bool>>>, version: u32) {
    let alignment_location = get_alignment_location(version);

    let alignment_pattern = vec![
        vec![true, true, true, true, true],
        vec![true, false, false, false, true],
        vec![true, false, true, false, true],
        vec![true, false, false, false, true],
        vec![true, true, true, true, true],
    ];

    for (x, y) in alignment_location { // center

        if matrix[x as usize][y as usize] != None {
            continue;
        }

        for i in 0..5 {
            for j in 0..5 {
                matrix[x as usize - 2 + i][y as usize - 2 + j] = Some(alignment_pattern[i][j]);
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

    let x = 7;
    let y = 4 * version + 10;
    
    matrix[y as usize][x as usize] = Some(true);
    
}



fn get_dimension(version: u32) -> u32 {
    DIMENSION[version as usize - 1]
}

fn get_alignment_location(version: u32) -> Vec<(u32, u32)> {
    let mut alignment_pattern = Vec::new();

    if version == 1 {
        return alignment_pattern;
    }

    for i in 0..ALIGNMENT_PATTERN[version as usize - 2].len() {
        for j in 0..ALIGNMENT_PATTERN[version as usize - 2].len() {
            alignment_pattern.push((ALIGNMENT_PATTERN[version as usize - 2][i], ALIGNMENT_PATTERN[version as usize - 2][j]));
        }
    }

    println!("{:?}", alignment_pattern);
    alignment_pattern
}

const DIMENSION: [u32; 40] = [
    21, 25, 29, 33, 37, 41, 45, 49, 53, 57,
    61, 65, 69, 73, 77, 81, 85, 89, 93, 97,
    101, 105, 109, 113, 117, 121, 125, 129, 133, 137,
    141, 145, 149, 153, 157, 161, 165, 169, 173, 177
];

const ALIGNMENT_PATTERN: [&[u32]; 39] = [
    &[6, 18], &[6, 22], &[6, 26], &[6, 30], &[6, 34], &[6, 22, 38], &[6, 24, 42], &[6, 26, 46], &[6, 28, 50], &[6, 30, 54],
    &[6, 32, 58], &[6, 34, 62], &[6, 26, 46, 66], &[6, 26, 48, 70], &[6, 26, 50, 74], &[6, 30, 54, 78], &[6, 30, 56, 82], &[6, 30, 58, 86], &[6, 34, 62, 90], &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98], &[6, 30, 54, 78, 102], &[6, 28, 54, 80, 106], &[6, 32, 58, 84, 110], &[6, 30, 58, 86, 114], &[6, 34, 62, 90, 118], &[6, 26, 50, 74, 98, 122], &[6, 30, 54, 78, 102, 126], &[6, 26, 52, 78, 104, 130], &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138], &[6, 30, 58, 86, 114, 142], &[6, 34, 62, 90, 118, 146], &[6, 30, 54, 78, 102, 126, 150], &[6, 24, 50, 76, 102, 128, 154], &[6, 28, 54, 80, 106, 132, 158], &[6, 32, 58, 84, 110, 136, 162], &[6, 26, 54, 82, 110, 138, 166], &[6, 30, 58, 86, 114, 142, 170]
];