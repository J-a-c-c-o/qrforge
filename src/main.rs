mod encode;
mod correction;

fn main() {
    let version = 1;
    let error_correction = "M";
    let mode = "alphanumeric";
    let text = "HELLO WORLD";
    
    let combined_data = encode::encode(version, error_correction, mode, text);

    let correction = correction::correction(version, error_correction, combined_data.clone());

    for i in 0..combined_data.len() {
        print!("{}", if combined_data[i] { "1" } else { "0" });
        if (i + 1) % 8 == 0 {
            print!(" ");
        }
    }
    
}