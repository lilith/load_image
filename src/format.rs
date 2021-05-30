/// File type of the image
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Format {
    Unknown,
    Jpeg,
    Png,
}

impl Default for Format {
    #[inline(always)]
    fn default() -> Self {
        Format::Unknown
    }
}
