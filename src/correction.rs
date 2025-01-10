pub fn correction(version: u32, error_correction: &str, combined_data: Vec<bool>) -> Vec<bool> {

    let (blocks_group1, blocks_group2) = split_into_blocks(combined_data, version, error_correction);


    // nice print
    for i in 0..blocks_group1.len() {
        println!("Group 1 Block {}", i + 1);
        for j in 0..blocks_group1[i].len() {
            for k in 0..blocks_group1[i][j].len() {
                print!("{}", if blocks_group1[i][j][k] { "1" } else { "0" });
            }
            print!(" ");
        }
        println!();
    }

    for i in 0..blocks_group2.len() {
        println!("Group 2 Block {}", i + 1);
        for j in 0..blocks_group2[i].len() {
            for k in 0..blocks_group2[i][j].len() {
                print!("{}", if blocks_group2[i][j][k] { "1" } else { "0" });
            }
            print!(" ");
        }
        println!();
    }

    println!();

    Vec::new()
}


// [[[blocksg1, amountg1, blocksg2, amountg2], ...], ...]
const BLOCK_LOOKUP: [[[u32; 4]; 4]; 40] = [
    // Version 1
    [[1, 19, 0, 0], [1, 16, 0, 0], [1, 13, 0, 0], [1, 9, 0, 0]],
    // Version 2
    [[1, 34, 0, 0], [1, 28, 0, 0], [1, 22, 0, 0], [1, 16, 0, 0]],
    // Version 3
    [[1, 55, 0, 0], [1, 44, 0, 0], [2, 17, 0, 0], [2, 13, 0, 0]],
    // Version 4
    [[1, 80, 0, 0], [2, 32, 0, 0], [2, 24, 0, 0], [4, 9, 0, 0]],
    // Version 5
    [[1, 108, 0, 0], [2, 43, 0, 0], [2, 15, 2, 16], [2, 11, 2, 12]],
    // Version 6
    [[2, 68, 0, 0], [4, 27, 0, 0], [4, 19, 0, 0], [4, 15, 0, 0]],
    // Version 7
    [[2, 78, 0, 0], [4, 31, 0, 0], [2, 14, 4, 15], [4, 13, 1, 14]],
    // Version 8
    [[2, 97, 0, 0], [2, 38, 2, 39], [4, 18, 2, 19], [4, 14, 2, 15]],
    // Version 9
    [[2, 116, 0, 0], [3, 36, 2, 37], [4, 16, 4, 17], [4, 12, 4, 13]],
    // Version 10
    [[2, 68, 2, 69], [4, 43, 1, 44], [6, 19, 2, 20], [6, 15, 2, 16]],
    // Version 11
    [[4, 81, 0, 0], [1, 50, 4, 51], [4, 22, 4, 23], [3, 12, 8, 13]],
    // Version 12
    [[2, 92, 2, 93], [6, 36, 2, 37], [4, 20, 6, 21], [7, 14, 4, 15]],
    // Version 13
    [[4, 107, 0, 0], [8, 37, 1, 38], [8, 20, 4, 21], [12, 11, 4, 12]],
    // Version 14
    [[3, 115, 1, 116], [4, 40, 5, 41], [11, 16, 5, 17], [11, 12, 5, 13]],
    // Version 15
    [[5, 87, 1, 88], [5, 41, 5, 42], [5, 24, 7, 25], [11, 12, 7, 13]],
    // Version 16
    [[5, 98, 1, 99], [7, 45, 3, 46], [15, 19, 2, 20], [3, 15, 13, 16]],
    // Version 17
    [[1, 107, 5, 108], [10, 46, 1, 47], [1, 22, 15, 23], [2, 14, 17, 15]],
    // Version 18
    [[5, 120, 1, 121], [9, 43, 4, 44], [17, 22, 1, 23], [2, 14, 19, 15]],
    // Version 19
    [[3, 113, 4, 114], [3, 44, 11, 45], [17, 21, 4, 22], [9, 13, 16, 14]],
    // Version 20
    [[3, 107, 5, 108], [3, 41, 13, 42], [15, 24, 5, 25], [15, 15, 10, 16]],
    // Version 21
    [[4, 116, 4, 117], [17, 42, 0, 0], [17, 22, 6, 23], [19, 16, 6, 17]],
    // Version 22
    [[2, 111, 7, 112], [17, 46, 0, 0], [7, 24, 16, 25], [34, 13, 0, 0]],
    // Version 23
    [[4, 121, 5, 122], [4, 47, 14, 48], [11, 24, 14, 25], [16, 15, 14, 16]],
    // Version 24
    [[6, 117, 4, 118], [6, 45, 14, 46], [11, 24, 16, 25], [30, 16, 0, 0]],
    // Version 25
    [[8, 106, 4, 107], [8, 47, 13, 48], [7, 24, 22, 25], [22, 16, 8, 17]],
    // Version 26
    [[10, 114, 2, 115], [19, 46, 4, 47], [28, 22, 6, 23], [33, 16, 4, 17]],
    // Version 27
    [[8, 122, 4, 123], [22, 45, 3, 46], [8, 23, 26, 24], [12, 15, 28, 16]],
    // Version 28
    [[3, 117, 10, 118], [3, 45, 23, 46], [4, 24, 31, 25], [11, 15, 31, 16]],
    // Version 29
    [[7, 116, 7, 117], [21, 45, 7, 46], [1, 23, 37, 24], [19, 15, 26, 16]],
    // Version 30
    [[5, 115, 10, 116], [19, 47, 10, 48], [15, 24, 25, 25], [23, 15, 25, 16]],
    // Version 31
    [[13, 115, 3, 116], [2, 46, 29, 47], [42, 24, 1, 25], [23, 15, 28, 16]],
    // Version 32
    [[17, 115, 0, 0], [10, 46, 23, 47], [10, 24, 35, 25], [19, 15, 35, 16]],
    // Version 33
    [[17, 115, 1, 116], [14, 46, 21, 47], [29, 24, 19, 25], [11, 15, 46, 16]],
    // Version 34
    [[13, 115, 6, 116], [14, 46, 23, 47], [44, 24, 7, 25], [59, 16, 1, 17]],
    // Version 35
    [[12, 121, 7, 122], [12, 47, 26, 48], [39, 24, 14, 25], [22, 15, 41, 16]],
    // Version 36
    [[6, 121, 14, 122], [6, 47, 34, 48], [46, 24, 10, 25], [2, 15, 64, 16]],
    // Version 37
    [[17, 122, 4, 123], [29, 46, 14, 47], [49, 24, 10, 25], [24, 15, 46, 16]],
    // Version 38
    [[4, 122, 18, 123], [13, 46, 32, 47], [48, 24, 14, 25], [42, 15, 32, 16]],
    // Version 39
    [[20, 117, 4, 118], [40, 47, 7, 48], [43, 24, 22, 25], [10, 15, 67, 16]],
    // Version 40
    [[19, 118, 6, 119], [18, 47, 31, 48], [34, 24, 34, 25], [20, 15, 61, 16]],
];

fn split_into_blocks(mut combined_data: Vec<bool>, version: u32, error_correction: &str) -> (Vec<Vec<Vec<bool>>>, Vec<Vec<Vec<bool>>>) {

    
    let correction_level = match error_correction {
        "L" => 0,
        "M" => 1,
        "Q" => 2,
        "H" => 3,
        _ => 0,
    };

    let block_lookup = BLOCK_LOOKUP[version as usize - 1][correction_level];

    // group 1
    let mut blocks_group1: Vec<Vec<Vec<bool>>> = Vec::new();
    let group1_blocks = block_lookup[0] as usize;
    let group1_amount = block_lookup[1] as usize;

    for _ in 0..group1_blocks {
        let mut block: Vec<Vec<bool>> = Vec::new();
        for _ in 0..group1_amount {
            let mut data: Vec<bool> = Vec::new();
            for _ in 0..8 {
                data.push(combined_data.remove(0));
            }

            block.push(data);
        }
        blocks_group1.push(block);
    }
    

    // group 2
    let mut blocks_group2: Vec<Vec<Vec<bool>>> = Vec::new();
    let group2_blocks = block_lookup[2] as usize;
    let group2_amount = block_lookup[3] as usize;

    for _ in 0..group2_blocks {
        let mut block: Vec<Vec<bool>> = Vec::new();
        for _ in 0..group2_amount {
            let mut data: Vec<bool> = Vec::new();
            for _ in 0..8 {
                data.push(combined_data.remove(0));
            }
            block.push(data);
        }
        blocks_group2.push(block);
    }

    (blocks_group1, blocks_group2)
}
    