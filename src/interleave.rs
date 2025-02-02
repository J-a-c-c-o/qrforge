pub fn interleave(blocks: Vec<Vec<Vec<bool>>>, ec_blocks: Vec<Vec<Vec<bool>>>, version: usize) -> Vec<bool> {
    let interleave_ec = interleave_blocks(&ec_blocks);
    let interleave_data = interleave_blocks(&blocks);

    let total_capacity = interleave_data.iter().map(|b| b.len()).sum::<usize>()
        + interleave_ec.iter().map(|b| b.len()).sum::<usize>()
        + REMAINING_BITS[version as usize - 1] as usize;

    let mut result: Vec<bool> = Vec::with_capacity(total_capacity);
    for block in &interleave_data {
        result.extend_from_slice(block);
    }
    for block in &interleave_ec {
        result.extend_from_slice(block);
    }

    let remainder_bits = get_remainder_bits(version);
    result.extend_from_slice(&remainder_bits);
    result
}

fn interleave_blocks(blocks: &Vec<Vec<Vec<bool>>>) -> Vec<Vec<bool>> {
    let mut result: Vec<Vec<bool>> = Vec::new();
    let max_length = blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    result.reserve(max_length * blocks.len());

    for i in 0..max_length {
        for block in blocks {
            if let Some(data) = block.get(i) {
                result.push(data.clone());
            }
        }
    }
    result
}

fn get_remainder_bits(version: usize) -> Vec<bool> {
    let mut result: Vec<bool> = Vec::new();
    let remaining = REMAINING_BITS[version - 1];
    for _ in 0..remaining {
        result.push(false);
    }
    result
}


const REMAINING_BITS: [u32; 40] = [
    0,7,7,7,7,7,0,0,0,0,0,0,0,3,3,3,3,3,3,3,4,4,4,4,4,4,4,3,3,3,3,3,3,3,0,0,0,0,0,0];