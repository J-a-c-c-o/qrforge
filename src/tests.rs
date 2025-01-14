use test::Bencher;
    use super::*;

    #[bench]
    fn test(b: &mut Bencher) {
        b.iter(||{
        let version = 40;
        let error_correction = "H";
        let mode = "alphanumeric";
        let text = "HELLO WORLD";
        
        let combined_data = encode::encode(version, error_correction, mode, text);
    
        let (blocks, ec_blocks) = correction::correction_interleave(version, error_correction, combined_data.clone());
    
        let result = interleave::interleave(blocks, ec_blocks, version);
    
    
        let matrix = matrix_builder::build_qr_matrix(version, error_correction, result);
        });
        
    }