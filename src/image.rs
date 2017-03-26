use lodepng;
use bitmap::*;
use rgb::*;

pub type GRAY8 = lodepng::Grey<u8>;
pub type GRAY16 = lodepng::Grey<u16>;
pub type GRAYA8 = lodepng::GreyAlpha<u8>;
pub type GRAYA16 = lodepng::GreyAlpha<u16>;

#[derive(Debug)]
pub enum Image {
    RGB8(Bitmap<RGB8>),
    RGBA8(Bitmap<RGBA8>),
    RGB16(Bitmap<RGB16>),
    RGBA16(Bitmap<RGBA16>),
    GRAY8(Bitmap<GRAY8>),
    GRAY16(Bitmap<GRAY16>),
    GRAYA8(Bitmap<GRAYA8>),
    GRAYA16(Bitmap<GRAYA16>),
}
