#![cfg(feature = "svg")]
use std::{fs::File, io::Write};

use crate::{error::QRError, qrcode::QRCode};

#[derive(PartialEq)]
enum ErrorEnum {
    InvalidBorder,
    InvalidWidth,
    InvalidHeight,
}

/// Represents an RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
    /// Alpha component.
    pub a: u8,
}

impl Color {
    /// Creates a new `Color` with the given red, green, blue, and alpha values.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
}

/// SvgQRCode builds SVG files.
///
/// # Examples
///
/// Generate an SVG from a QRCode and save it:
///
/// ```rust
/// # use qr_module::{QRCode, Mode, ErrorCorrection};
/// let qr = QRCode::builder()
///     .add_segment(Some(Mode::Byte), b"https://example.com")
///     .error_correction(ErrorCorrection::L)
///     .version(1.into())
///     .build()
///     .unwrap();
///
/// qr.image_builder()  // you can either add a svg method or use SvgQRCode::new
///     .set_border(10)
///     .set_width(300)
///     .set_height(300)
///     .build_svg_file("output.svg")
///     .unwrap();
/// ```
pub struct SvgQRCode {
    qr_code: QRCode,
    width: usize,
    height: usize,
    border: usize,
    border_color: Color,
    dark_color: Color,
    light_color: Color,
    error: Vec<ErrorEnum>,
}

impl SvgQRCode {
    /// Creates a new SvgQRCode with default parameters based on the QR code's dimensions.
    pub(crate) fn new(qr_code: QRCode) -> Self {
        let dimension = qr_code.dimension();
        SvgQRCode {
            qr_code,
            width: dimension,
            height: dimension,
            border: 0,
            border_color: Color::new(255, 255, 255, 255),
            dark_color: Color::new(0, 0, 0, 255),
            light_color: Color::new(255, 255, 255, 255),
            error: Vec::new(),
        }
    }

    /// Sets the border size.
    ///
    /// If the provided border is too large such that the drawable area is smaller than
    /// the QR code dimension, an error is recorded.
    pub fn set_border(&mut self, border: usize) -> &mut Self {
        if self.width - 2 * border < self.qr_code.dimension()
            || self.height - 2 * border < self.qr_code.dimension()
        {
            self.error.push(ErrorEnum::InvalidBorder);
        } else {
            self.error.retain(|e| *e != ErrorEnum::InvalidBorder);
            self.border = border;
        }
        self
    }

    /// Sets the image width.
    ///
    /// The width must be at least as large as the QR code dimension.
    pub fn set_width(&mut self, width: usize) -> &mut Self {
        if width < self.qr_code.dimension() {
            self.error.push(ErrorEnum::InvalidWidth);
        } else {
            self.error.retain(|e| *e != ErrorEnum::InvalidWidth);
            self.width = width;
        }
        self
    }

    /// Sets the image height.
    ///
    /// The height must be at least as large as the QR code dimension.
    pub fn set_height(&mut self, height: usize) -> &mut Self {
        if height < self.qr_code.dimension() {
            self.error.push(ErrorEnum::InvalidHeight);
        } else {
            self.error.retain(|e| *e != ErrorEnum::InvalidHeight);
            self.height = height;
        }
        self
    }

    /// Sets the color used for the image border.
    pub fn set_border_color(&mut self, color: Color) -> &mut Self {
        self.border_color = color;
        self
    }

    /// Sets the color used for dark QR code modules.
    pub fn set_dark_color(&mut self, color: Color) -> &mut Self {
        self.dark_color = color;
        self
    }

    /// Sets the color used for light QR code modules.
    pub fn set_light_color(&mut self, color: Color) -> &mut Self {
        self.light_color = color;
        self
    }

    // Similarly update other setter methods to use Color::new(...) when needed.

    /// Builds the SVG content as a byte vector.
    ///
    /// Returns an error if any of the parameters are invalid.
    pub fn build_svg_bytes(&self) -> Result<Vec<u8>, QRError> {
        if !self.error.is_empty() {
            return Err(QRError::new("Invalid parameters"));
        }

        let pixel_size_width = (self.width - 2 * self.border) / self.qr_code.dimension();
        let pixel_size_height = (self.height - 2 * self.border) / self.qr_code.dimension();
        let pixel_size = std::cmp::min(pixel_size_width, pixel_size_height);

        let border_width = (self.width - self.qr_code.dimension() * pixel_size) / 2;
        let border_height = (self.height - self.qr_code.dimension() * pixel_size) / 2;

        // let mut document = Document::new()
        //     .set("width", self.width.to_string())
        //     .set("height", self.height.to_string())
        //     .set("viewBox", format!("0 0 {} {}", self.width, self.height));

        // // Add background.
        // let background = Rectangle::new()
        //     .set("width", "100%")
        //     .set("height", "100%")
        //     .set(
        //         "fill",
        //         format!(
        //             "rgba({}, {}, {}, {})",
        //             self.border_color.r,
        //             self.border_color.g,
        //             self.border_color.b,
        //             self.border_color.a as f32 / 255.0
        //         ),
        //     );
        // document = document.add(background);

        // // Add QR code modules.
        // for y in 0..self.qr_code.dimension() {
        //     for x in 0..self.qr_code.dimension() {
        //         if self.qr_code.get(x, y) {
        //             let module = Rectangle::new()
        //                 .set("x", x * pixel_size + border_width)
        //                 .set("y", y * pixel_size + border_height)
        //                 .set("width", pixel_size)
        //                 .set("height", pixel_size)
        //                 .set(
        //                     "fill",
        //                     format!(
        //                         "rgba({}, {}, {}, {})",
        //                         self.dark_color.r,
        //                         self.dark_color.g,
        //                         self.dark_color.b,
        //                         self.dark_color.a as f32 / 255.0
        //                     ),
        //                 );
        //             document = document.add(module);
        //         }
        //     }
        // }

        // Ok(document.to_string().into_bytes())

        let mut svg = Vec::new();
        svg.push(b'<');
        svg.extend_from_slice(b"svg xmlns=\"http://www.w3.org/2000/svg\" ");
        svg.extend_from_slice(format!("width=\"{}\" ", self.width).as_bytes());
        svg.extend_from_slice(format!("height=\"{}\" ", self.height).as_bytes());
        svg.extend_from_slice(
            format!("viewBox=\"0 0 {} {}\">\n", self.width, self.height).as_bytes(),
        );

        // Add background.
        svg.push(b'<');
        svg.extend_from_slice(b"rect ");
        svg.extend_from_slice(format!("width=\"{}\" ", self.width).as_bytes());
        svg.extend_from_slice(format!("height=\"{}\" ", self.height).as_bytes());
        svg.extend_from_slice(b"fill=\"");
        svg.extend_from_slice(
            format!(
                "rgba({}, {}, {}, {})",
                self.border_color.r,
                self.border_color.g,
                self.border_color.b,
                self.border_color.a as f32 / 255.0
            )
            .as_bytes(),
        );
        svg.extend_from_slice(b"\" />\n");

        // Add QR code modules.
        for y in 0..self.qr_code.dimension() {
            for x in 0..self.qr_code.dimension() {
                if self.qr_code.get(x, y) {
                    svg.push(b'<');
                    svg.extend_from_slice(b"rect ");
                    svg.extend_from_slice(
                        format!("x=\"{}\" ", x * pixel_size + border_width).as_bytes(),
                    );
                    svg.extend_from_slice(
                        format!("y=\"{}\" ", y * pixel_size + border_height).as_bytes(),
                    );
                    svg.extend_from_slice(format!("width=\"{}\" ", pixel_size).as_bytes());
                    svg.extend_from_slice(format!("height=\"{}\" ", pixel_size).as_bytes());
                    svg.extend_from_slice(b"fill=\"");
                    svg.extend_from_slice(
                        format!(
                            "rgba({}, {}, {}, {})",
                            self.dark_color.r,
                            self.dark_color.g,
                            self.dark_color.b,
                            self.dark_color.a as f32 / 255.0
                        )
                        .as_bytes(),
                    );
                    svg.extend_from_slice(b"\" />\n");
                }
            }
        }

        svg.push(b'<');
        svg.extend_from_slice(b"/svg>");
        Ok(svg)
    }

    /// Builds the SVG and writes it to a file at the given path.
    ///
    /// Returns an error if the file cannot be created or written to.
    pub fn build_svg_file(&self, path: &str) -> Result<(), QRError> {
        let svg_data = self.build_svg_bytes()?;
        let mut file = File::create(path).map_err(|e| QRError::new(&e.to_string()))?;
        file.write_all(&svg_data)
            .map_err(|e| QRError::new(&e.to_string()))?;
        Ok(())
    }

    /// Constructs a color from red, green, blue, and alpha components.
    pub fn color(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        Color::new(red, green, blue, alpha)
    }

    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
}
