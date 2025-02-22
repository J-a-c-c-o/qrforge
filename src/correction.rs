use lazy_static::lazy_static;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::{
    constants::{BLOCK_LOOKUP, EC_CODEWORDS},
    ErrorCorrection,
};

lazy_static! {
    static ref GF_TABLES: ([u8; 256], [u8; 256]) = generate_gf_tables();
}

/// Perform error correction on the data
pub(crate) fn correction(
    version: usize,
    error_correction: &ErrorCorrection,
    combined_data: Vec<bool>,
) -> (Vec<Vec<Vec<bool>>>, Vec<Vec<Vec<bool>>>) {
    let blocks: Vec<Vec<Vec<bool>>> = split_into_blocks(combined_data, version, error_correction);
    let ec_codewords = ec_codewords(version, error_correction);

    // Use pre-generated tables
    let (antilog_table, log_table) = &*GF_TABLES;

    // Pre-generate generator polynomial once
    let generator = generate_generator_polynomial(ec_codewords, log_table, antilog_table);

    // Process blocks in parallel
    #[cfg(feature = "parallel")]
    let ec_blocks: Vec<Vec<Vec<bool>>> = blocks
        .par_iter()
        .map(|block| {
            let polynomial = build_polynomial(block);
            let result = part0(
                ec_codewords,
                &generator,
                &polynomial,
                log_table,
                antilog_table,
            );

            // Pre-allocate result vectors
            result
                .par_iter()
                .map(|&code| {
                    let mut data = Vec::with_capacity(8);
                    (0..8).for_each(|k| {
                        data.push((code & (1 << (7 - k))) != 0);
                    });
                    data
                })
                .collect()
        })
        .collect();

    // Process blocks sequentially
    #[cfg(not(feature = "parallel"))]
    let ec_blocks: Vec<Vec<Vec<bool>>> = blocks
        .iter()
        .map(|block| {
            let polynomial = build_polynomial(block);
            let result = part0(
                ec_codewords,
                &generator,
                &polynomial,
                log_table,
                antilog_table,
            );

            // Pre-allocate result vectors
            result
                .iter()
                .map(|&code| {
                    let mut data = Vec::with_capacity(8);
                    (0..8).for_each(|k| {
                        data.push((code & (1 << (7 - k))) != 0);
                    });
                    data
                })
                .collect()
        })
        .collect();

    (blocks, ec_blocks)
}

/// Build polynomial from data
fn build_polynomial(data: &[Vec<bool>]) -> Vec<(u32, u32)> {
    let mut polynomial = Vec::with_capacity(data.len());
    let size = (data.len() - 1) as u32;

    for (i, byte) in data.iter().enumerate() {
        let value = byte.iter().enumerate().fold(0, |acc, (j, &bit)| {
            acc + if bit { 2u32.pow(7 - j as u32) } else { 0 }
        });
        polynomial.push((value, size - i as u32));
    }
    polynomial
}

/// Generate generator polynomial
fn generate_generator_polynomial(
    ec_codewords: u32,
    log_table: &[u8; 256],
    antilog_table: &[u8; 256],
) -> Vec<(u32, u32)> {
    // Start with X^1
    let mut polynomial = vec![(0, 1)]; // (coefficient α^0, exponent 1)

    // Multiply by (X + α^i) ec_codewords times
    for i in 0..ec_codewords {
        polynomial = multiply_polynomial(&polynomial, i, log_table, antilog_table);
    }

    polynomial.sort_by(|a, b| b.1.cmp(&a.1));
    polynomial
}

/// Multiply two polynomials
fn multiply_polynomial(
    polynomial: &[(u32, u32)],
    alpha_power: u32,
    log_table: &[u8; 256],
    antilog_table: &[u8; 256],
) -> Vec<(u32, u32)> {
    // Collect expanded terms
    let mut result_temp = Vec::with_capacity(polynomial.len() * 2);
    for &(coeff, exp) in polynomial {
        // Multiply with (0, 1) => shift exponent by 1
        result_temp.push((coeff % 255, exp + 1));
        // Multiply with (alpha^alpha_power, 0)
        result_temp.push(((coeff + alpha_power) % 255, exp));
    }

    // Combine like exponents
    let mut result = Vec::new();
    for (val, e) in result_temp {
        if let Some(existing) = result.iter_mut().find(|(_, ex)| *ex == e) {
            let tmp = lookup(
                lookup(val, antilog_table) ^ lookup(existing.0, antilog_table),
                log_table,
            );
            existing.0 = tmp;
        } else {
            result.push((val, e));
        }
    }
    result
}

/// first step of creating error correction codewords
fn part0(
    n: u32,
    generator: &[(u32, u32)],
    data_polynomial: &[(u32, u32)],
    log_table: &[u8; 256],
    antilog_table: &[u8; 256],
) -> Vec<u32> {
    let mut polynomial: Vec<(u32, u32)> = Vec::new();
    for (a, b) in data_polynomial.iter() {
        polynomial.push((*a, (*b) + n));
    }
    let mut generator_polynomial: Vec<(u32, u32)> = Vec::new();
    let diff = polynomial.first().unwrap().1 - generator.first().unwrap().1;

    for (a, b) in generator.iter() {
        generator_polynomial.push((*a, (*b) + diff));
    }

    partn(
        &polynomial,
        &generator_polynomial,
        data_polynomial.len() as u32,
        log_table,
        antilog_table,
    )
}

/// Recursive step of creating error correction codewords
fn partn(
    polynomial: &[(u32, u32)],
    generator: &[(u32, u32)],
    n: u32,
    log_table: &[u8; 256],
    antilog_table: &[u8; 256],
) -> Vec<u32> {
    if n == 0 {
        return polynomial.iter().map(|(a, _)| *a).collect();
    }

    let lookup_value = lookup(polynomial[0].0, log_table);

    // Pre-allocate new polynomial with max possible size
    let mut new_poly = Vec::with_capacity(max(polynomial.len(), generator.len()));

    // Combine generator transformation and XOR operations
    for i in 1..max(polynomial.len(), generator.len()) {
        let poly_val = polynomial.get(i).map_or(0, |&(a, _)| a);
        let gen_val = if i < generator.len() {
            lookup((generator[i].0 + lookup_value) % 255, antilog_table)
        } else {
            0
        };

        new_poly.push((poly_val ^ gen_val, polynomial[0].1 - i as u32));
    }

    partn(&new_poly, generator, n - 1, log_table, antilog_table)
}

/// Find maximum of two usize values
fn max(a: usize, b: usize) -> usize {
    if a > b {
        return a;
    }
    b
}

/// Perform a lookup in the log table
fn lookup(a: u32, log_table: &[u8; 256]) -> u32 {
    log_table[a as usize] as u32
}

/// Generate Galois Field tables
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

/// Split data into correct sized blocks
fn split_into_blocks(
    combined_data: Vec<bool>,
    version: usize,
    error_correction: &ErrorCorrection,
) -> Vec<Vec<Vec<bool>>> {
    let correction_level = error_correction.to_value();

    let block_lookup = BLOCK_LOOKUP[version - 1][correction_level];

    let group1_blocks = block_lookup[0] as usize;
    let group1_amount = block_lookup[1] as usize;
    let group2_blocks = block_lookup[2] as usize;
    let group2_amount = block_lookup[3] as usize;

    let codewords: Vec<Vec<bool>> = combined_data.chunks(8).map(|c| c.to_vec()).collect();

    let mut blocks = Vec::new();
    let mut offset = 0;

    for _ in 0..group1_blocks {
        blocks.push(codewords[offset..offset + group1_amount].to_vec());
        offset += group1_amount;
    }
    for _ in 0..group2_blocks {
        blocks.push(codewords[offset..offset + group2_amount].to_vec());
        offset += group2_amount;
    }

    blocks
}

/// Get the number of error correction codewords
fn ec_codewords(version: usize, error_correction: &ErrorCorrection) -> u32 {
    let correction_level = error_correction.to_value();

    EC_CODEWORDS[version - 1][correction_level] as u32
}
