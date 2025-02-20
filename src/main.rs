//! This module is the entry point to the QR code library.
//! It defines the main QRCode data structure along with a builder (QRBuilder)
//! for creating QR codes. It also demonstrates how to generate images or apply
//! structured append encoding.
//!
//! # Examples
//!
//! Creating a QR code:
//!
//! ```rust
//! let qr = QRCode::builder()
//!     .add_segment(Some(Mode::Byte), b"Hello world")
//!     .error_correction(ErrorCorrection::L)
//!     .version(Version::V(1))
//!     .build()?;
//! ```
//!
//! Generating a PNG image:
//!
//! ```rust
//! qr.image_builder()
//!     .set_width(200)
//!     .set_height(200)
//!     .set_border(4)
//!     .build_image_file("hello.png")?;
//! ```
//!
//! Generating an SVG image:
//!
//! ```rust
//! qr.svg_builder()
//!     .set_width(200)
//!     .set_height(200)
//!     .set_border(4)
//!     .build_image_file("hello.svg")?;
//! ```
//!
//! Creating a QR code with structured append:
//!
//! ```rust
//! let qr = QRCode::builder()
//!     .add_segment(Some(Mode::Byte), b"I read the newspaper")
//!     .add_segment(Some(Mode::Numeric), b"1234567890")
//!     .error_correction(ErrorCorrection::L)
//!     .version(Version::V(1))
//!     .build_with_structual_append()?;
//! ```
//!

mod constants;
mod correction;
mod encode;
mod enums;
mod error;
mod image;
mod interleave;
mod matrix_builder;
mod matrix_builder_micro;
mod mode_selector;
mod qrcode;
mod qrcode_builder;
mod utils;
use enums::{ErrorCorrection, Mode, Version};
use error::QRError;
use qrcode::QRCode;

fn main() -> Result<(), QRError> {
    let start = std::time::Instant::now();

    // Japanese text "こんにちは" (Hello) in Shift-JIS encoding;
    let kanji = vec![0x93, 0xfa, 0x96, 0x7b, 0x82, 0xcc, 0x82, 0xc1, 0x82, 0xbd];
    let utf8: Vec<u8> = vec![255, 61];
    let bytes = b"Hello world";
    // Generate some numeric data for a QR code.
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

    for (index, qr_code) in structured.iter().enumerate() {
        qr_code
            .image_builder()
            .set_width(200)
            .set_height(200)
            .set_border(4)
            .build_image_file(&format!("hello_{}.png", index))?;
    }
    println!("QR Code generated in: {:?}", start.elapsed());

    Ok(())
}
