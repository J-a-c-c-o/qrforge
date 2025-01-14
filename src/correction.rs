pub fn correction_interleave(version: u32, error_correction: &str, combined_data: Vec<bool>) -> (Vec<Vec<Vec<bool>>>, Vec<Vec<Vec<bool>>>) {

    let blocks = split_into_blocks(combined_data, version, error_correction);

    let ec_codewords = ec_codewords(version, error_correction);
    

    let mut ec_blocks: Vec<Vec<Vec<bool>>>= Vec::new();

    let (antilog_table, log_table) = generate_gf_tables();

    for i in 0..blocks.len() {
        let polynomial = build_polynomial(blocks.get(i).unwrap().clone());
        let generator = generate_generator_polynomial(ec_codewords, &log_table, &antilog_table);
        let result = part0(ec_codewords, &generator, &polynomial, &log_table, &antilog_table);

        let mut ec_block: Vec<Vec<bool>> = Vec::new();
        for j in 0..result.len() {
            let mut data: Vec<bool> = Vec::new();
            for k in 0..8 {
                data.push((result[j] & (1 << (7-k))) != 0);
            }
            ec_block.push(data);
        }

        ec_blocks.push(ec_block);
    }



    (blocks, ec_blocks)

}

fn build_polynomial(data: Vec<Vec<bool>>) -> Vec<(u32,u32)> {
    let mut polynomial: Vec<(u32,u32)> = Vec::new();
    let size = data.len() as u32 - 1;
    for i in 0..data.len() {
        let mut value = 0;
        for j in 0..data[i].len() {
            if data[i][j] {
                value += 2u32.pow(7-j as u32);
            }

        }
        polynomial.push((value, size-i as u32));
    }


    polynomial
}



fn generate_generator_polynomial(ec_codewords: u32, log_table: &[u8; 256], antilog_table: &[u8; 256]) -> Vec<(u32,u32)> {

    let mut generator = recursive_generator((ec_codewords-1) as usize, vec![(0,1), (0,0)], 1, log_table, antilog_table);
    
    generator.sort_by(|a, b| b.1.cmp(&a.1));

    generator
    
}


fn recursive_generator(n: usize, polynomial: Vec<(u32,u32)>, a: u32, log_table: &[u8; 256], antilog_table: &[u8; 256]) -> Vec<(u32,u32)> {
    if n == 0 {
        return polynomial;
    }

    let multiplier = vec![(0,1), (a,0)];
    let mut new_polynomial_temp: Vec<(u32,u32)> = Vec::new();

    // multiply
    for (a,b) in polynomial.iter() {
        for (c,d) in multiplier.iter() {
            new_polynomial_temp.push(((a+c)%255, b+d));
        }
    }

    // combine
    let mut new_polynomial: Vec<(u32,u32)> = Vec::new();
    for (a,b) in new_polynomial_temp.iter() {
        let mut found = false;
        for (c,d) in new_polynomial.iter_mut() {
            if d == b {
                let temp = lookup(reverse_lookup(*a,antilog_table) ^ reverse_lookup(*c,antilog_table), log_table);
                *c = temp;
                found = true;
                break;
            }
        }
        if !found {
            new_polynomial.push((*a,*b));
        }
    }




    recursive_generator(n-1, new_polynomial, a+1, log_table, antilog_table)

    
}



fn part0(n: u32, generator: &Vec<(u32,u32)>, data_polynomial: &Vec<(u32,u32)>, log_table: &[u8; 256], antilog_table: &[u8; 256]) -> Vec<u32> {
    let mut polynomial: Vec<(u32,u32)> = Vec::new();
    for (a,b) in data_polynomial.iter() {
        polynomial.push((*a,(*b)+n));
    }
    let mut generator_polynomial: Vec<(u32,u32)> = Vec::new();
    let diff = polynomial.get(0).unwrap().1 - generator.get(0).unwrap().1;

    for (a,b) in generator.iter() {
        generator_polynomial.push((*a,(*b)+diff));
    }

    partn(&polynomial, &generator_polynomial, data_polynomial.len() as u32, log_table, antilog_table)
    
    
}

fn partn(polynomial: &Vec<(u32,u32)>, generator: &Vec<(u32,u32)>, n: u32, log_table: &[u8; 256], antilog_table: &[u8; 256]) -> Vec<u32> {
    if n == 0 {
        let mut result: Vec<u32> = Vec::new();
        for (a,_) in polynomial.iter() {
            result.push(*a);
        }
        return result;
    }

    // step a
    let lookup = lookup(polynomial.get(0).unwrap().0, log_table);
    let mut temp: Vec::<(u32,u32)> = Vec::new();
    for (a,b) in generator.iter() {
        temp.push((((*a) + lookup) % 255, *b));
    }

    // convert to polynomial
    let mut generator_polynomial: Vec<(u32,u32)> = Vec::new();
    for (a,b) in temp.iter() {
        let new_a = reverse_lookup(*a, antilog_table);
        generator_polynomial.push((new_a, *b));
    }


    // step b
    let mut new_poly: Vec<(u32,u32)> = Vec::new();
    for i in 0..max(polynomial.len(), generator_polynomial.len()) {
        let mut poly: &(u32, u32) = &(0,polynomial.get(0).unwrap().1 - i as u32);
        // polynomial.get(i as usize).unwrap();
        if polynomial.len() > i as usize {
            poly = polynomial.get(i as usize).unwrap();
        }
        
        let mut gen = 0;

        // generator_polynomial.get(i as usize).unwrap().0;
        if generator_polynomial.len() > i as usize {
            gen = generator_polynomial.get(i as usize).unwrap().0;
        }

        new_poly.push((poly.0 ^ gen, poly.1));

    }


    // remove 0s leading
    new_poly.remove(0);

    // recursive
    partn(&new_poly , generator, n-1, log_table, antilog_table)

    
}

fn max(a: usize, b: usize) -> usize {
    if a > b {
        return a;
    }
    b
}


fn lookup(a: u32, log_table: &[u8; 256]) -> u32 {

    

    log_table[a as usize] as u32
    
}


fn reverse_lookup(a: u32, antilog_table: &[u8; 256]) -> u32 {
    antilog_table[a as usize] as u32
}


fn generate_gf_tables() -> ([u8; 256], [u8; 256]) {
    let primitive_polynomial: u16 = 285;
    let mut antilog_table = [0u8; 256];
    let mut log_table = [0u8; 256];

    let mut value: u16 = 1;

    for i in 0..255 {
        antilog_table[i] = value as u8;
        log_table[value as usize] = i as u8;

        value <<= 1;
        if value & 0x100 != 0 {
            value ^= primitive_polynomial;
        }
    }
    antilog_table[255] = antilog_table[0];
    (antilog_table, log_table)
}


fn split_into_blocks(mut combined_data: Vec<bool>, version: u32, error_correction: &str) -> Vec<Vec<Vec<bool>>> {

    
    let correction_level = match error_correction {
        "L" => 0,
        "M" => 1,
        "Q" => 2,
        "H" => 3,
        _ => 0,
    };

    let block_lookup = BLOCK_LOOKUP[version as usize - 1][correction_level];

    // group 1
    let mut blocks: Vec<Vec<Vec<bool>>> = Vec::new();
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
        blocks.push(block);
    }
    

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
        blocks.push(block);
    }

    blocks
}




fn ec_codewords(version: u32, error_correction: &str) -> u32 {
    let correction_level = match error_correction {
        "L" => 0,
        "M" => 1,
        "Q" => 2,
        "H" => 3,
        _ => 0,
    };

    EC_CODEWORDS[version as usize - 1][correction_level] as u32
}


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

const EC_CODEWORDS: [[usize; 4]; 40] = [
    // Version 1-10
    [7, 10, 13, 17], [10, 16, 22, 28], [15, 26, 18, 22], [20, 18, 26, 16],
    [26, 24, 18, 22], [18, 16, 24, 28], [20, 18, 18, 26], [24, 22, 22, 26],
    [30, 22, 20, 24], [18, 26, 24, 28],

    // Version 11-20
    [20, 30, 28, 24], [24, 22, 26, 28], [26, 22, 24, 22], [30, 24, 20, 24],
    [22, 24, 30, 24], [24, 28, 24, 30], [28, 28, 28, 28], [30, 26, 28, 28],
    [28, 26, 26, 26], [28, 26, 30, 28],

    // Version 21-30
    [28, 26, 28, 30], [28, 28, 30, 24], [30, 28, 30, 30], [30, 28, 30, 30],
    [26, 28, 28, 30], [28, 28, 28, 28], [30, 28, 28, 30], [30, 28, 28, 30],
    [30, 28, 28, 30], [30, 28, 28, 30],

    // Version 31-40
    [30, 28, 28, 30], [30, 28, 28, 30], [30, 28, 28, 30], [30, 28, 28, 30],
    [30, 28, 28, 30], [30, 28, 28, 30], [30, 28, 28, 30], [30, 28, 28, 30],
    [30, 28, 28, 30], [30, 28, 28, 30]
];


