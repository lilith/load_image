pub const GRAY: &[u8] = include_bytes!("gray.icc");

#[cfg(any(feature = "jpeg", feature = "mozjpeg"))]
pub const CMYK: &[u8] = include_bytes!("cmyk.icc");

#[cfg(any(feature = "jpeg", feature = "mozjpeg"))]
pub const ADOBE1998: &[u8] = include_bytes!("AdobeRGB1998.icc");
