#![feature(test)]

extern crate test;

mod encode;
mod correction;
mod matrix_builder;
mod interleave;


#[cfg(test)]
mod tests;


fn main() {
    let version = 6;
    let error_correction = "H";
    let mode = "alphanumeric";
    let text = "HELLO WORLD";
  
    let combined_data = encode::encode(version, error_correction, mode, text);

    let (blocks, ec_blocks) = correction::correction(version, error_correction, combined_data.clone());

    let result = interleave::interleave(blocks, ec_blocks, version);


    let matrix = matrix_builder::build_qr_matrix(version as usize, error_correction, result);

    matrix.pretty_print();
    
}