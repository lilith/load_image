pub const GRAY: &[u8] = include_bytes!("gray.icc");
#[cfg(feature = "jpeg")]
pub const CMYK: &[u8] = include_bytes!("cmyk.icc");
#[cfg(feature = "jpeg")]
pub const ADOBE1998: &[u8] = include_bytes!("AdobeRGB1998.icc");
