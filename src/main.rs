mod encode;
mod correction;
mod matrix_builder;
mod interleave;


fn main() {
    let version = 25;
    let error_correction = "Q";
    let mode = "alphanumeric";
    let text = "HELLO WORLD";
    
    let combined_data = encode::encode(version, error_correction, mode, text);

    let (blocks, ec_blocks) = correction::correction_interleave(version, error_correction, combined_data.clone());

    let result = interleave::interleave(blocks, ec_blocks, version);


    let matrix = matrix_builder::build_qr_matrix(version, error_correction, result);


    pretty_print(&matrix);
    
}

fn pretty_print(matrix: &Vec<Vec<Option<bool>>>) {
    let dimension = matrix.len();
    for i in 0..dimension as usize {
        for j in 0..dimension as usize {
            if matrix[i][j] == None {
                //make it green
                print!("🟩");
            } else {
                print!("{}", if matrix[i][j].unwrap() { "⬛" } else { "⬜" });
            }
        }
        println!();
    }

    println!();

}