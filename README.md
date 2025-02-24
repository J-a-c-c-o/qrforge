
# QRForge

QRForge is a QR code generator written in Rust. It supports generating QR codes in both raster (PNG) and vector (SVG) formats. The library provides a flexible builder pattern for creating QR codes with various parameters such as version, error correction level, and data segments.

## Features

- Generate QR codes in PNG and SVG formats
- Support for different QR code versions and error correction levels
- Structured append for splitting data across multiple QR codes
- Optional parallel processing with Rayon

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
qrforge = "x.y.z"
```

To enable optional features:

```toml
[dependencies]
qrforge = { version = "x.y.z", features = ["image", "svg", "parallel"] }
```

## Usage

### Basic Usage

```rust
use qrforge::{QRCode, Mode, ErrorCorrection};

let qr = QRCode::builder()
    .add_segment(Some(Mode::Byte), b"Hello world")
    .error_correction(ErrorCorrection::L)
    .version(1.into())
    .build()
    .unwrap();
```

### Generating a PNG Image

```rust
qr.image_builder()
    .set_width(200)
    .set_height(200)
    .set_border(4)
    .build_image_file("hello.png")
    .unwrap();
```

### Generating an SVG Image

```rust
qr.svg_builder()
    .set_width(200)
    .set_height(200)
    .set_border(4)
    .build_svg_file("hello.svg")
    .unwrap();
```

### Structured Append

```rust
let qr_codes = QRCode::builder()
    .add_segment(Some(Mode::Byte), b"I read the newspaper")
    .add_segment(Some(Mode::Numeric), b"1234567890")
    .error_correction(ErrorCorrection::L)
    .version(Version::V(1))
    .build_with_structual_append()
    .unwrap();
```

## Examples

Examples can be found in the examples directory. To run an example, use the following command:

```sh
cargo run --example qrgen_image --features image
cargo run --example qrgen_svg --features svg
```

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
