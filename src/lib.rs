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

#[cfg(feature = "avif")]
mod avif;

pub use crate::convert::FromOptions;
pub use crate::format::*;
pub use crate::image::*;
pub use crate::loader::*;
pub use lodepng::Error as Error;
use std::path::Path;

/// Load image from file path
#[inline]
pub fn load_image(path: impl AsRef<Path>, opaque: bool) -> Result<Image, Error> {
    Loader::new().opaque(opaque).load_path(path)
}

/// Load image from file data in memory
#[inline]
pub fn load_image_data(data: &[u8], opaque: bool) -> Result<Image, Error> {
    Loader::new().opaque(opaque).load_data(data)
}
