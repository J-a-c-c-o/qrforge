use qrforge::QRCode;
use qrforge::QRError;
use qrforge::{ErrorCorrection, Mode, Version};

fn main() -> Result<(), QRError> {
    // Japanese text "こんにちは" (Hello) in Shift-JIS encoding;
    let kanji = vec![0x93, 0xfa, 0x96, 0x7b, 0x82, 0xcc, 0x82, 0xc1, 0x82, 0xbd];
    let bytes = b"Hello world";
    let mut numbers = vec![];
    for i in 0..35 {
        numbers.push((i % 10) as u8 + 48);
    }

    let structured = QRCode::builder()
        .add_segment(Some(Mode::Byte), b"I read the newspaper")
        .add_segment(Some(Mode::Numeric), b"1234567890")
        .error_correction(ErrorCorrection::L)
        .version(Version::V(1))
        .build_with_structual_append()?;

    for qr in structured {
        qr.print();
    }

    let qr = QRCode::builder()
        .add_segment(Some(Mode::Byte), bytes)
        .add_segment(Some(Mode::Kanji), &kanji)
        .add_segment(Some(Mode::Numeric), &numbers)
        .error_correction(ErrorCorrection::L)
        .version(Version::V(5))
        .build()?;

    qr.print();

    Ok(())
}
