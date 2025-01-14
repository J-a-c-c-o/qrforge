pub fn interleave(blocks: Vec<Vec<Vec<bool>>>, ec_blocks: Vec<Vec<Vec<bool>>>, version: u32) -> Vec<bool> {
    let interleave_ec = interleave_blocks(&ec_blocks);
    let interleave_data = interleave_blocks(&blocks);

    let mut result: Vec<bool> = Vec::new();
    for block in interleave_data.iter() {
        result.extend_from_slice(block);
    }

    for block in interleave_ec.iter() {
        result.extend_from_slice(block);
    }
    
    // fill remaining bits
    let remainder_bits = get_remainder_bits(version);

    result.extend_from_slice(&remainder_bits);

    result
}


fn interleave_blocks(blocks: &Vec<Vec<Vec<bool>>>) -> Vec<Vec<bool>> {
    let mut result: Vec<Vec<bool>> = Vec::new();
    let mut max_length = 0;
    for block in blocks.iter() {
        if block.len() > max_length {
            max_length = block.len();
        }
    }

    for i in 0..max_length {
        for block in blocks.iter() {
            if block.len() > i {
                result.push(block.get(i).unwrap().clone());
            }
        }
    }

    result
}

fn get_remainder_bits(version: u32) -> Vec<bool> {
    let mut result: Vec<bool> = Vec::new();
    let remaining = REMAINING_BITS[version as usize - 1];
    for _ in 0..remaining {
        result.push(false);
    }
    result
}


const REMAINING_BITS: [u32; 40] = [
    0,7,7,7,7,7,0,0,0,0,0,0,0,3,3,3,3,3,3,3,4,4,4,4,4,4,4,3,3,3,3,3,3,3,0,0,0,0,0,0];