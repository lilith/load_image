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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Rotate {
    FlipX,
    D90,
    D90FlipX,
    D180,
    D180FlipX,
    D270,
    D270FlipX,
}

impl Image {
    pub fn rotated(&self, r: Rotate) -> Image {
        match *self {
            Image::RGB8(ref bitmap) => Image::RGB8(Self::rotated_bitmap(bitmap, r)),
            Image::RGBA8(ref bitmap) => Image::RGBA8(Self::rotated_bitmap(bitmap, r)),
            Image::RGB16(ref bitmap) => Image::RGB16(Self::rotated_bitmap(bitmap, r)),
            Image::RGBA16(ref bitmap) => Image::RGBA16(Self::rotated_bitmap(bitmap, r)),
            Image::GRAY8(ref bitmap) => Image::GRAY8(Self::rotated_bitmap(bitmap, r)),
            Image::GRAY16(ref bitmap) => Image::GRAY16(Self::rotated_bitmap(bitmap, r)),
            Image::GRAYA8(ref bitmap) => Image::GRAYA8(Self::rotated_bitmap(bitmap, r)),
            Image::GRAYA16(ref bitmap) => Image::GRAYA16(Self::rotated_bitmap(bitmap, r)),
        }
    }

    fn rotated_bitmap<T: Copy>(bitmap: &Bitmap<T>, rotation: Rotate) -> Bitmap<T> {
        let (width, height) = (bitmap.width, bitmap.height);
        let s = &bitmap.bitmap;
        let mut d = Vec::with_capacity(bitmap.width * bitmap.height);
        match rotation {
            Rotate::FlipX => {
                for y in 0..height {
                    for x in (0..width).rev() {
                        d.push(s[x + y * width]);
                    }
                }
                Bitmap::new(d, width, height)
            },
            Rotate::D90 => {
                for x in (0..width).rev() {
                    for y in 0..height {
                        d.push(s[x + y * width]);
                    }
                }
                Bitmap::new(d, height, width)
            },
            Rotate::D90FlipX => {
                for x in (0..width).rev() {
                    for y in (0..height).rev() {
                        d.push(s[x + y * width]);
                    }
                }
                Bitmap::new(d, height, width)
            },
            Rotate::D180 => {
                for y in (0..height).rev() {
                    for x in (0..width).rev() {
                        d.push(s[x + y * width]);
                    }
                }
                Bitmap::new(d, width, height)
            },
            Rotate::D180FlipX => {
                for y in (0..height).rev() {
                    for x in 0..width {
                        d.push(s[x + y * width]);
                    }
                }
                Bitmap::new(d, width, height)
            },
            Rotate::D270 => {
                for x in 0..width {
                    for y in (0..height).rev() {
                        d.push(s[x + y * width]);
                    }
                }
                Bitmap::new(d, height, width)
            },
            Rotate::D270FlipX => {
                for x in 0..width {
                    for y in 0..height {
                        d.push(s[x + y * width]);
                    }
                }
                Bitmap::new(d, height, width)
            },
        }
    }
}
