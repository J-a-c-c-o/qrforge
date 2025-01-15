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
    
        let (blocks, ec_blocks) = correction::correction(version, error_correction, combined_data.clone());
    
        let result = interleave::interleave(blocks, ec_blocks, version);
    
    
        let _matrix = matrix_builder::build_qr_matrix(version as usize, error_correction, result);
        });
        
    }