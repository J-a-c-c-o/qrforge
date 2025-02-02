
use tokio::{self, task};

mod encode;
mod correction;
mod matrix_builder;
mod interleave;


#[tokio::main]
async fn main() {
    let version = 40;
    let error_correction = "H";
    let mode = "byte";
    let text = "HELLO WORLD";
      
    let prepared_matrix = task::spawn(matrix_builder::prepare_qr_matrix(version as usize));

    let combined_data = encode::encode(version, error_correction, mode, text);

    let (blocks, ec_blocks) = correction::correction(version, error_correction, combined_data);


    let result = interleave::interleave(blocks, ec_blocks, version);


    let mut matrix = prepared_matrix.await.unwrap();

    matrix_builder::build_qr_matrix(&mut matrix, version as usize, error_correction, result);

    matrix.pretty_print();   
    
}