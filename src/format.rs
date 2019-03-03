#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Format {
    Unknown,
    Jpeg,
    Png,
}

impl Default for Format {
    fn default() -> Self {
        Format::Unknown
    }
}
