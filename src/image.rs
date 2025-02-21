#![cfg(feature = "image")]
use image::{ImageBuffer, Rgba};

use crate::{error::QRError, qrcode::QRCode};

#[derive(PartialEq)]
enum ErrorEnum {
    InvalidBorder,
    InvalidWidth,
    InvalidHeight,
}

/// ImageQRCode builds raster image files (PNG, etc.).
///
/// # Examples
///
/// Generate an image from a QRCode and save it:
///
/// ```rust
/// # use qr_module::{QRCode, Mode, ErrorCorrection};
/// let qr = QRCode::builder()
///     .add_segment(Some(Mode::Byte), b"Hello world")
///     .error_correction(ErrorCorrection::L)
///     .version(1.into())
///     .build()
///     .unwrap();
///
/// qr.image_builder()
///     .set_border(10)
///     .set_width(300)
///     .set_height(300)
///     .build_image_file("output.png")
///     .unwrap();
/// ```
pub struct ImageQRCode {
    qr_code: QRCode,
    width: usize,
    height: usize,
    border: usize,
    border_color: Rgba<u8>,
    dark_color: Rgba<u8>,
    light_color: Rgba<u8>,
    error: Vec<ErrorEnum>,
}

impl ImageQRCode {
    /// Creates a new ImageQRCode with default parameters based on the QR code's dimensions.
    pub(crate) fn new(qr_code: QRCode) -> Self {
        let dimension = qr_code.dimension();
        ImageQRCode {
            qr_code,
            width: dimension,
            height: dimension,
            border: 0,
            border_color: Rgba([255, 255, 255, 255]),
            dark_color: Rgba([0, 0, 0, 255]),
            light_color: Rgba([255, 255, 255, 255]),
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
    pub fn set_border_color(&mut self, color: Rgba<u8>) -> &mut Self {
        self.border_color = color;
        self
    }

    /// Sets the color used for dark QR code modules.
    pub fn set_dark_color(&mut self, color: Rgba<u8>) -> &mut Self {
        self.dark_color = color;
        self
    }

    /// Sets the color used for light QR code modules.
    pub fn set_light_color(&mut self, color: Rgba<u8>) -> &mut Self {
        self.light_color = color;
        self
    }

    /// Builds an image buffer for the QR code.
    ///
    /// Returns an error if any of the parameters are invalid.
    pub fn build_image(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, QRError> {
        if !self.error.is_empty() {
            return Err(QRError::new("Invalid parameters"));
        }

        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);

        let pixel_size_width = (self.width - 2 * self.border) / self.qr_code.dimension();
        let pixel_size_height = (self.height - 2 * self.border) / self.qr_code.dimension();
        let pixel_size = std::cmp::min(pixel_size_width, pixel_size_height);

        let border_width = (self.width - self.qr_code.dimension() * pixel_size) / 2;
        let border_height = (self.height - self.qr_code.dimension() * pixel_size) / 2;

        // Draw background.
        for y in 0..self.height {
            for x in 0..self.width {
                img.put_pixel(x as u32, y as u32, self.border_color);
            }
        }

        // Draw QR code modules.
        for y in 0..self.qr_code.dimension() {
            for x in 0..self.qr_code.dimension() {
                let color = if self.qr_code.get(x, y) {
                    self.dark_color
                } else {
                    self.light_color
                };
                for i in 0..pixel_size {
                    for j in 0..pixel_size {
                        img.put_pixel(
                            (x * pixel_size + i + border_width) as u32,
                            (y * pixel_size + j + border_height) as u32,
                            color,
                        );
                    }
                }
            }
        }

        Ok(img)
    }

    /// Builds the image and saves it as a file at the given path.
    ///
    /// Returns an error if the image cannot be saved.
    pub fn build_image_file(&self, path: &str) -> Result<(), QRError> {
        let img = self.build_image()?;
        img.save(path).map_err(|e| QRError::new(&e.to_string()))?;
        Ok(())
    }

    /// Constructs a color using the given red, green, blue, and alpha components.
    pub fn color(red: u8, green: u8, blue: u8, alpha: u8) -> Rgba<u8> {
        Rgba([red, green, blue, alpha])
    }

    pub const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
    pub const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
    pub const RED: Rgba<u8> = Rgba([255, 0, 0, 255]);
    pub const GREEN: Rgba<u8> = Rgba([0, 255, 0, 255]);
    pub const BLUE: Rgba<u8> = Rgba([0, 0, 255, 255]);
    pub const YELLOW: Rgba<u8> = Rgba([255, 255, 0, 255]);
    pub const CYAN: Rgba<u8> = Rgba([0, 255, 255, 255]);
    pub const MAGENTA: Rgba<u8> = Rgba([255, 0, 255, 255]);
    pub const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);
}
