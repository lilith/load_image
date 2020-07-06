use crate::convert::*;
use crate::format::*;
use imgref::*;
use rgb::alt::*;
use rgb::*;
use std::fs;
#[cfg(feature = "stat")]
use std::io;
#[cfg(feature = "stat")]
use std::time;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImageMeta {
    pub format: Format,
    #[cfg(feature="stat")]
    pub created: u64,
    #[cfg(feature="stat")]
    pub modified: u64,
}

impl ImageMeta {
    #[cfg(not(feature="stat"))]
    pub fn new(format: Format, _: Option<fs::Metadata>) -> Self {
        ImageMeta {
            format,
        }
    }

    #[cfg(feature="stat")]
    pub fn new(format: Format, fs_meta: Option<fs::Metadata>) -> Self {
        fn time(t: Result<time::SystemTime, io::Error>) -> Option<u64> {
            t.ok().and_then(|d|d.duration_since(time::UNIX_EPOCH).ok()).map(|d| d.as_secs())
        }
        ImageMeta {
            format,
            created: fs_meta.as_ref().and_then(|stat| time(stat.created())).unwrap_or(1500000001),
            modified: fs_meta.and_then(|stat| time(stat.modified())).unwrap_or(1501296258),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub meta: ImageMeta,
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
    pub fn is_opaque(&self) -> bool {
        match self.bitmap {
            ImageData::RGB8(_) => true,
            ImageData::RGBA8(_) => false,
            ImageData::RGB16(_) => true,
            ImageData::RGBA16(_) => false,
            ImageData::GRAY8(_) => true,
            ImageData::GRAY16(_) => true,
            ImageData::GRAYA8(_) => false,
            ImageData::GRAYA16(_) => false,
        }
    }

    pub fn rotated(&self, r: Rotate) -> Image {
        let meta = self.meta.clone();
        match self.bitmap {
            ImageData::RGB8(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
            ImageData::RGBA8(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
            ImageData::RGB16(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
            ImageData::RGBA16(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAY8(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAY16(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAYA8(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAYA16(ref bitmap) => Self::from_opts(Self::rotated_bitmap(ImgRef::new(bitmap, self.width, self.height), r), meta),
        }
    }

    fn rotated_bitmap<T: Copy>(bitmap: ImgRef<T>, rotation: Rotate) -> ImgVec<T> {
        let (width, height, stride) = (bitmap.width(), bitmap.height(), bitmap.stride());
        let s = bitmap.buf();
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

#[cfg(feature="stat")]
#[test]
fn test_stat() {
    let file = fs::File::open("src/lib.rs").unwrap();
    let m = ImageMeta::new(Format::Unknown, file.metadata().ok());
    assert!(m.created >= 1499358961);
    assert!(m.modified >= 1499358961);
    assert!(m.modified >= m.created);
}
