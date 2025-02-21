//! # QR Code Generator
//!
//! This crate provides a QR code generator for encoding data into QR codes. The crate supports
//! encoding data in numeric, alphanumeric, byte, and kanji modes. It also supports structured
//! append for splitting data across multiple QR codes.
//!
//! The crate provides two main types: `QRCode` and `QRBuilder`. The `QRCode` type represents a
//! QR code and provides methods for generating images in PNG and SVG formats. The `QRBuilder`
//! type provides a builder pattern for creating QR codes with specific parameters.
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
mod interleave;
mod matrix_builder;
mod matrix_builder_micro;
mod mode_selector;
mod utils;

pub mod enums;
pub use enums::{ErrorCorrection, Mode, Version};

pub mod error;
pub use error::QRError;

pub mod qrcode;
pub use qrcode::QRCode;

pub mod qrcode_builder;
pub use qrcode_builder::QRBuilder;

pub mod image;
#[cfg(feature = "image")]
pub use image::ImageQRCode;

pub mod svg;
#[cfg(feature = "svg")]
pub use svg::SvgQRCode;
