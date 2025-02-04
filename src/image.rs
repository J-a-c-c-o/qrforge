use image::{ImageBuffer, Rgba};

use crate::{error::QRError, QRCode};

#[derive(PartialEq)]
enum ErrorEnum {
    InvalidBorder,
    InvalidWidth,
    InvalidHeight,
}

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
    pub(crate) fn new(qr_code: QRCode) -> Self {
        let width = qr_code.dimension();
        let height = qr_code.dimension();
        let border = 0;
        ImageQRCode {
            qr_code,
            width,
            height,
            border,
            border_color: Rgba([255, 255, 255, 255]),
            dark_color: Rgba([0, 0, 0, 255]),
            light_color: Rgba([255, 255, 255, 255]),
            error: Vec::new(),
        }
    }

    pub fn set_border(&mut self, border: usize) -> &mut Self {
        // border cannot reduce size of QR code to less than its dimension
        if self.width - 2 * border < self.qr_code.dimension() || self.height - 2 * border < self.qr_code.dimension() {
            self.error.push(ErrorEnum::InvalidBorder);
        } else {
            self.error.retain(|e| *e != ErrorEnum::InvalidBorder);
            self.border = border;
        }

        
        self
    }

    pub fn set_width(&mut self, width: usize) -> &mut Self {
        if width < self.qr_code.dimension() {
            self.error.push(ErrorEnum::InvalidWidth);
        } else {
            self.error.retain(|e| *e != ErrorEnum::InvalidWidth);
            self.width = width;
        }

        self
    }

    pub fn set_height(&mut self, height: usize) -> &mut Self {
        if height < self.qr_code.dimension() {
            self.error.push(ErrorEnum::InvalidHeight);
        } else {
            self.error.retain(|e| *e != ErrorEnum::InvalidHeight);
            self.height = height;
        }

        self
    }

    pub fn set_border_color(&mut self, color: Rgba<u8>) -> &mut Self {
        self.border_color = color;
        self
    }

    pub fn set_dark_color(&mut self, color: Rgba<u8>) -> &mut Self {
        self.dark_color = color;
        self
    }

    pub fn set_light_color(&mut self, color: Rgba<u8>) -> &mut Self {
        self.light_color = color;
        self
    }

    pub fn build(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, QRError> {
        // if there are errors, return an empty image
        if !self.error.is_empty() {
            return Err(QRError::new("Invalid parameters"));
        }

        


        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);
        
        let pixel_size_width = (self.width - 2 * self.border) / self.qr_code.dimension();
        let pixel_size_height = (self.height - 2 * self.border) / self.qr_code.dimension();

        let pixel_size = std::cmp::min(pixel_size_width, pixel_size_height);

        let border_width = (self.width - self.qr_code.dimension() * pixel_size) / 2;
        let border_height = (self.height - self.qr_code.dimension() * pixel_size) / 2;

        // draw background
        for y in 0..self.height {
            for x in 0..self.width {
                img.put_pixel(x as u32, y as u32, self.border_color);
            }
        }

        // draw QR code in the center
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