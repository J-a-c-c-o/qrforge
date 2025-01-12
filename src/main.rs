mod encode;
mod correction_interleave;

fn main() {
    let version = 1;
    let error_correction = "M";
    let mode = "alphanumeric";
    let text = "HELLO WORLD";
    
    let combined_data = encode::encode(version, error_correction, mode, text);
    let result: Vec<bool> = correction_interleave::correction_interleave(version, error_correction, combined_data.clone());

    // pretty print the result
    for i in 0..result.len() {
        if i % 8 == 0 {
            print!(" ");
        }
        if i % 64 == 0 {
            println!();
        }
        if result[i] {
            print!("1");
        } else {
            print!("0");
        }
    }



    
}