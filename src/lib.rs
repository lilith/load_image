mod alpha;
mod convert;
mod endian;
mod format;
mod image;
mod jpeg;
mod loader;
mod pixel_format;
mod png;
mod profiles;

pub use crate::convert::FromOptions;
pub use crate::format::*;
pub use crate::image::*;
pub use crate::loader::*;
use std::path::Path;

#[inline]
pub fn load_image<P: AsRef<Path>>(path: P, opaque: bool) -> Result<Image, lodepng::Error> {
    Loader::new().opaque(opaque).load_path(path)
}

#[inline]
pub fn load_image_data(data: &[u8], opaque: bool) -> Result<Image, lodepng::Error> {
    Loader::new().opaque(opaque).load_data(data)
}
