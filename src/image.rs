#![cfg(feature = "image")]
use image::{ImageBuffer, Rgba};

use crate::{color::Color, enums::ErrorEnum, error::QRError, qrcode::QRCode};

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
    border_color: Color,
    dark_color: Color,
    light_color: Color,
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
            self.error.push(ErrorEnum::Border);
        } else {
            self.error.retain(|e| *e != ErrorEnum::Border);
            self.border = border;
        }
        self
    }

    /// Sets the image width.
    ///
    /// The width must be at least as large as the QR code dimension.
    pub fn set_width(&mut self, width: usize) -> &mut Self {
        if width < self.qr_code.dimension() {
            self.error.push(ErrorEnum::Width);
        } else {
            self.error.retain(|e| *e != ErrorEnum::Width);
            self.width = width;
        }
        self
    }

    /// Sets the image height.
    ///
    /// The height must be at least as large as the QR code dimension.
    pub fn set_height(&mut self, height: usize) -> &mut Self {
        if height < self.qr_code.dimension() {
            self.error.push(ErrorEnum::Height);
        } else {
            self.error.retain(|e| *e != ErrorEnum::Height);
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

    /// Builds an image buffer for the QR code.
    ///
    /// Returns an error if any of the parameters are invalid.
    pub fn build_image(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, QRError> {
        if !self.error.is_empty() {
            return Err(QRError::new("Invalid parameters"));
        }

        let border_color = Rgba([
            self.border_color.r,
            self.border_color.g,
            self.border_color.b,
            self.border_color.a,
        ]);

        let dark_color = Rgba([
            self.dark_color.r,
            self.dark_color.g,
            self.dark_color.b,
            self.dark_color.a,
        ]);

        let light_color = Rgba([
            self.light_color.r,
            self.light_color.g,
            self.light_color.b,
            self.light_color.a,
        ]);

        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);

        let pixel_size_width = (self.width - 2 * self.border) / self.qr_code.dimension();
        let pixel_size_height = (self.height - 2 * self.border) / self.qr_code.dimension();
        let pixel_size = std::cmp::min(pixel_size_width, pixel_size_height);

        let border_width = (self.width - self.qr_code.dimension() * pixel_size) / 2;
        let border_height = (self.height - self.qr_code.dimension() * pixel_size) / 2;

        // Draw background.
        for y in 0..self.height {
            for x in 0..self.width {
                img.put_pixel(x as u32, y as u32, border_color);
            }
        }

        // Draw QR code modules.
        for y in 0..self.qr_code.dimension() {
            for x in 0..self.qr_code.dimension() {
                let color = if self.qr_code.get(x, y) {
                    dark_color
                } else {
                    light_color
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
}
