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
mod loader;
mod convert;
mod format;
mod alpha;

use std::path::Path;
pub use image::*;
pub use loader::*;
pub use format::*;

#[inline]
pub fn load_image<P: AsRef<Path>>(path: P, opaque: bool) -> Result<Image, lodepng::Error> {
    Loader::new().opaque(opaque).load_path(path)
}

#[inline]
pub fn load_image_data(data: &[u8], opaque: bool) -> Result<Image, lodepng::Error> {
    Loader::new().opaque(opaque).load_data(data)
}
