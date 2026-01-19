#![doc = include_str!("../README.md")]

mod alpha;
mod convert;
mod endian;
mod error;
mod format;
mod image;
mod loader;
mod pixel_format;
mod png;

pub use crate::error::*;

#[cfg(feature = "mozjpeg")]
mod mozjpeg;
#[cfg(all(not(feature = "mozjpeg"), feature = "jpeg"))]
mod jpeg;
#[cfg(any(feature = "mozjpeg", feature = "jpeg"))]
mod exif;
mod profiles;

#[cfg(feature = "avif")]
mod avif;

#[cfg(feature = "webp")]
mod webp;

pub use crate::convert::FromOptions;
pub use crate::format::*;
pub use crate::image::*;
pub use crate::loader::*;
use std::path::Path;

/// Load image from file path. Use [`Loader`] to configure it.
///
/// Pixels will be in sRGB color space. To load DisplayP3 or other, use [`Loader`].
pub fn load_path(path: impl AsRef<Path>) -> Result<Image, Error> {
    Loader::new().load_path(path)
}

/// Load image from file data in memory. Use [`Loader`] to configure it.
pub fn load_data(data: &[u8]) -> Result<Image, Error> {
    Loader::new().load_data(data)
}

#[doc(hidden)]
#[deprecated(note = "use load_path or Loader::new()")]
pub fn load_image(path: impl AsRef<Path>, opaque: bool) -> Result<Image, Error> {
    Loader::new().opaque(opaque).load_path(path)
}

#[doc(hidden)]
#[deprecated(note = "use load_data or Loader::new()")]
pub fn load_image_data(data: &[u8], opaque: bool) -> Result<Image, Error> {
    Loader::new().opaque(opaque).load_data(data)
}

/// Re-export of related crates
pub mod export {
    /// Re-export of the [`rgb`](https://lib.rs/crates/rgb) crate
    pub mod rgb {
        pub use ::rgb::alt::*;
        pub use ::rgb::*;
        pub use ::rgb::prelude::*;
        pub type GRAY16 = ::rgb::Gray<u16>;
        pub type GRAY8 = ::rgb::Gray<u8>;
        pub type GRAYA16 = ::rgb::GrayAlpha<u16>;
        pub type GRAYA8 = ::rgb::GrayAlpha<u8>;
    }

    /// Re-export of the [`imgref`](https://lib.rs/crates/imgref) crate
    pub mod imgref {
        use super::rgb;
        pub use imgref::{ImgRef, ImgVec};

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum ImgRefKind<'data> {
            RGB8(ImgRef<'data, rgb::RGB8>),
            RGBA8(ImgRef<'data, rgb::RGBA8>),
            RGB16(ImgRef<'data, rgb::RGB16>),
            RGBA16(ImgRef<'data, rgb::RGBA16>),
            GRAY8(ImgRef<'data, crate::export::rgb::GRAY8>),
            GRAY16(ImgRef<'data, crate::export::rgb::GRAY16>),
            GRAYA8(ImgRef<'data, crate::export::rgb::GRAYA8>),
            GRAYA16(ImgRef<'data, crate::export::rgb::GRAYA16>),
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum ImgVecKind {
            RGB8(ImgVec<rgb::RGB8>),
            RGBA8(ImgVec<rgb::RGBA8>),
            RGB16(ImgVec<rgb::RGB16>),
            RGBA16(ImgVec<rgb::RGBA16>),
            GRAY8(ImgVec<crate::export::rgb::GRAY8>),
            GRAY16(ImgVec<crate::export::rgb::GRAY16>),
            GRAYA8(ImgVec<crate::export::rgb::GRAYA8>),
            GRAYA16(ImgVec<crate::export::rgb::GRAYA16>),
        }
    }
}
