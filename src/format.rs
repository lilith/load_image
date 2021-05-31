/// File type of the image
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "avif", non_exhaustive)]
pub enum Format {
    Unknown,
    Jpeg,
    Png,
    #[cfg(feature = "avif")]
    Avif,
}

impl Default for Format {
    #[inline(always)]
    fn default() -> Self {
        Format::Unknown
    }
}
