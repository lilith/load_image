#![allow(unknown_lints)]

extern crate lodepng;
extern crate lcms2;
extern crate mozjpeg;
extern crate exif;
extern crate file;
extern crate rgb;
extern crate imgref;

mod pixel_format;
mod endian;
mod png;
mod jpeg;
mod image;
mod convert;

use std::io::Read;
use std::path::Path;
use rgb::*;
pub use image::*;

pub fn load_image<P: AsRef<Path>>(path: P, opaque: bool) -> Result<Image, lodepng::Error> {
    let path = path.as_ref();
    let data = if path.as_os_str() == "-" {
        let mut data = Vec::new();
        std::io::stdin().read_to_end(&mut data)?;
        data
    } else {
        file::get(path)?
    };
    load_image_data(&data, opaque)
}

pub fn load_image_data(data: &[u8], opaque: bool) -> Result<Image, lodepng::Error> {
    if data.starts_with(b"\x89PNG") {
        png::load_png(data, opaque)
    } else if data[0] == 0xFF {
        jpeg::load_jpeg(data)
    } else {
        Err(lodepng::Error(28))
    }
}
