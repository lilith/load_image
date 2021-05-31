/// File type of the image
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(any(feature = "avif", feature = "webp"), non_exhaustive)]
pub enum Format {
    Unknown,
    Jpeg,
    Png,
    #[cfg(feature = "avif")]
    Avif,
    #[cfg(feature = "webp")]
    WebP,
}

impl Default for Format {
    #[inline(always)]
    fn default() -> Self {
        Format::Unknown
    }
}
