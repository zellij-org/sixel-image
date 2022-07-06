//! This library provides an interface for querying, manipulating and serializing sixel data.
//!
//! There are several methods provided here to do this:
//!
//! 1. If you already have all the serialized sixel bytes, construct [`SixelImage`] directly
//! 2. If you'd like to parse bytes in real time "on the wire", use [`SixelDeserializer`]
//!
//! # Example
//! ```rust
//! use std::io::Read;
//! use std::io::BufReader;
//! use std::fs::File;
//! use sixel_image::SixelImage;
//! 
//! fn main() {
//!     let f = File::open("/home/aram/Downloads/lady-of-shalott.six").unwrap();
//!     let mut reader = BufReader::new(f);
//!     let mut buffer = Vec::new();
//!     reader.read_to_end(&mut buffer).unwrap();
//!     let sixel_image = SixelImage::new(&buffer).unwrap();
//!     let serialized = sixel_image.serialize();
//!     println!("{}", serialized);
//! }
//! ```

mod sixel_serializer;
mod sixel_deserializer;

pub use sixel_serializer::SixelSerializer;
pub use sixel_deserializer::SixelDeserializer;

use std::fmt;
use std::collections::BTreeMap;
use sixel_tokenizer::{ColorCoordinateSystem, Parser};

#[derive(Debug, Clone)]
pub struct SixelImage {
    color_registers: BTreeMap<u16, SixelColor>,
    pixels: Vec<Vec<Pixel>>,
}

impl SixelImage {
    /// Constructs a new `SixelImage` out of an existing slice of serialized sixel bytes
    pub fn new(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut parser = Parser::new();
        let mut sixel_deserializer = SixelDeserializer::new();
        for byte in bytes {
            let mut handle_result = Ok(());
            parser.advance(&byte, |sixel_event| {
                handle_result = sixel_deserializer.handle_event(sixel_event);
            });
            handle_result?
        }
        let sixel_image = sixel_deserializer.create_image();
        sixel_image
    }
    /// Returns the (height, width) of the image in pixels
    pub fn pixel_size(&self) -> (usize, usize) { // (height, width) in pixels
        let width = self.pixels.first().map(|first_line| first_line.len()).unwrap_or(0);
        let height = self.pixels.len();
        (height, width)
    }
    /// Serializes the whole image, returning a stringified sixel representation of it
    pub fn serialize(&self) -> String {
        let sixel_serializer = SixelSerializer::new(&self.color_registers, &self.pixels);
        let serialized_image = sixel_serializer.serialize();
        serialized_image
    }
    /// Serializes a specific rectangle of this image without manipulating the image itself
    pub fn serialize_range(&self, start_x_index: usize, start_y_index: usize, width: usize, height: usize) -> String {
        let sixel_serializer = SixelSerializer::new(&self.color_registers, &self.pixels);
        let serialized_image = sixel_serializer.serialize_range(start_x_index, start_y_index, width, height);
        serialized_image
    }
    /// Manipulates the image in-place, cutting out a rectangle with the specified coordinates. If
    /// the rectangle exceeds the image, it will be partially cut out.
    pub fn cut_out(&mut self, start_x_index: usize, start_y_index: usize, width: usize, height: usize) {
        for row in self.pixels.iter_mut().skip(start_y_index).take(height) {
            for pixel in row.iter_mut().skip(start_x_index).take(width) {
                pixel.on = false;
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Pixel {
    on: bool,
    color: u16, 
}

impl fmt::Debug for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.on {
            write!(f, "{}", self.color)
        } else {
            write!(f, "x")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SixelColor {
    Rgb(u8, u8, u8), // 0-100
    Hsl(u16, u8, u8), // 0-360, 0-100, 0-100
}

impl From<ColorCoordinateSystem> for SixelColor {
    fn from(item: ColorCoordinateSystem) -> Self {
        match item {
            ColorCoordinateSystem::HLS(x, y, z) => SixelColor::Hsl(x as u16, y as u8, z as u8),
            ColorCoordinateSystem::RGB(x, y, z) => SixelColor::Rgb(x as u8, y as u8, z as u8),
        }
    }
}

#[cfg(test)]
mod tests;
