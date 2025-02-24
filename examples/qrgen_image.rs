use qrforge::Color;
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

    let qr = QRCode::builder()
        .add_segment(Some(Mode::Byte), bytes)
        .add_segment(Some(Mode::Kanji), &kanji)
        .add_segment(Some(Mode::Numeric), &numbers)
        .error_correction(ErrorCorrection::L)
        .version(Version::V(5))
        .build()?;

    qr.image_builder()
        .set_dark_color(Color::new(0, 0, 0, 255))
        .set_light_color(Color::new(255, 255, 255, 255))
        .set_width(200)
        .set_height(200)
        .set_border(4)
        .build_image_file("hello.png")?;

    Ok(())
}
