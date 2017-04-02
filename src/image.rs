use lodepng;
use imgref::*;
use rgb::*;

pub type GRAY8 = lodepng::Grey<u8>;
pub type GRAY16 = lodepng::Grey<u16>;
pub type GRAYA8 = lodepng::GreyAlpha<u8>;
pub type GRAYA16 = lodepng::GreyAlpha<u16>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub bitmap: ImageData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageData {
    RGB8(Vec<RGB8>),
    RGBA8(Vec<RGBA8>),
    RGB16(Vec<RGB16>),
    RGBA16(Vec<RGBA16>),
    GRAY8(Vec<GRAY8>),
    GRAY16(Vec<GRAY16>),
    GRAYA8(Vec<GRAYA8>),
    GRAYA16(Vec<GRAYA16>),
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
        match self.bitmap {
            ImageData::RGB8(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
            ImageData::RGBA8(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
            ImageData::RGB16(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
            ImageData::RGBA16(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
            ImageData::GRAY8(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
            ImageData::GRAY16(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
            ImageData::GRAYA8(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
            ImageData::GRAYA16(ref bitmap) => Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r).into(),
        }
    }

    fn rotated_bitmap<T: Copy>(bitmap: ImgRef<T>, rotation: Rotate) -> ImgVec<T> {
        let (width, height, stride) = (bitmap.width(), bitmap.height(), bitmap.stride);
        let s = &bitmap.buf;
        let mut d = Vec::with_capacity(bitmap.width() * bitmap.height());
        match rotation {
            Rotate::FlipX => {
                for y in 0..height {
                    for x in (0..width).rev() {
                        d.push(s[x + y * stride]);
                    }
                }
                ImgVec::new(d, width, height)
            },
            Rotate::D90 => {
                for x in (0..width).rev() {
                    for y in 0..height {
                        d.push(s[x + y * stride]);
                    }
                }
                ImgVec::new(d, height, width)
            },
            Rotate::D90FlipX => {
                for x in (0..width).rev() {
                    for y in (0..height).rev() {
                        d.push(s[x + y * stride]);
                    }
                }
                ImgVec::new(d, height, width)
            },
            Rotate::D180 => {
                for y in (0..height).rev() {
                    for x in (0..width).rev() {
                        d.push(s[x + y * stride]);
                    }
                }
                ImgVec::new(d, width, height)
            },
            Rotate::D180FlipX => {
                for y in (0..height).rev() {
                    for x in 0..width {
                        d.push(s[x + y * stride]);
                    }
                }
                ImgVec::new(d, width, height)
            },
            Rotate::D270 => {
                for x in 0..width {
                    for y in (0..height).rev() {
                        d.push(s[x + y * stride]);
                    }
                }
                ImgVec::new(d, height, width)
            },
            Rotate::D270FlipX => {
                for x in 0..width {
                    for y in 0..height {
                        d.push(s[x + y * stride]);
                    }
                }
                ImgVec::new(d, height, width)
            },
        }
    }
}
