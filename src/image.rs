use crate::convert::*;
use crate::format::*;
use std::fs;
#[cfg(feature = "stat")]
use std::io;
#[cfg(feature = "stat")]
use std::time;

/// Re-export of the [`rgb`](https://lib.rs/crates/rgb) crate
pub mod rgb {
    pub use rgb::*;
    pub use rgb::alt::*;
}

/// Re-export of the [`imgref`](https://lib.rs/crates/imgref) crate
pub mod imgref {
    pub use imgref::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ImgRefKind<'data> {
        RGB8(ImgRef<'data, crate::rgb::RGB8>),
        RGBA8(ImgRef<'data, crate::rgb::RGBA8>),
        RGB16(ImgRef<'data, crate::rgb::RGB16>),
        RGBA16(ImgRef<'data, crate::rgb::RGBA16>),
        GRAY8(ImgRef<'data, crate::rgb::GRAY8>),
        GRAY16(ImgRef<'data, crate::rgb::GRAY16>),
        GRAYA8(ImgRef<'data, crate::rgb::GRAYA8>),
        GRAYA16(ImgRef<'data, crate::rgb::GRAYA16>),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ImgVecKind {
        RGB8(ImgVec<crate::rgb::RGB8>),
        RGBA8(ImgVec<crate::rgb::RGBA8>),
        RGB16(ImgVec<crate::rgb::RGB16>),
        RGBA16(ImgVec<crate::rgb::RGBA16>),
        GRAY8(ImgVec<crate::rgb::GRAY8>),
        GRAY16(ImgVec<crate::rgb::GRAY16>),
        GRAYA8(ImgVec<crate::rgb::GRAYA8>),
        GRAYA16(ImgVec<crate::rgb::GRAYA16>),
    }
}

use crate::imgref::{ImgRef, ImgRefKind, ImgVec, ImgVecKind};

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
    RGB8(Vec<crate::rgb::RGB8>),
    RGBA8(Vec<crate::rgb::RGBA8>),
    RGB16(Vec<crate::rgb::RGB16>),
    RGBA16(Vec<crate::rgb::RGBA16>),
    GRAY8(Vec<crate::rgb::GRAY8>),
    GRAY16(Vec<crate::rgb::GRAY16>),
    GRAYA8(Vec<crate::rgb::GRAYA8>),
    GRAYA16(Vec<crate::rgb::GRAYA16>),
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
    /// True if pixel format doesn't support alpha. This function doesn't check pixels.
    #[inline]
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

    /// Returns an enum with `Img<&[Pixel]>`
    #[inline]
    pub fn as_imgref(&self) -> ImgRefKind<'_> {
        match self.bitmap {
            ImageData::RGB8(ref bitmap) => ImgRefKind::RGB8(ImgRef::new(bitmap, self.width, self.height)),
            ImageData::RGBA8(ref bitmap) => ImgRefKind::RGBA8(ImgRef::new(bitmap, self.width, self.height)),
            ImageData::RGB16(ref bitmap) => ImgRefKind::RGB16(ImgRef::new(bitmap, self.width, self.height)),
            ImageData::RGBA16(ref bitmap) => ImgRefKind::RGBA16(ImgRef::new(bitmap, self.width, self.height)),
            ImageData::GRAY8(ref bitmap) => ImgRefKind::GRAY8(ImgRef::new(bitmap, self.width, self.height)),
            ImageData::GRAY16(ref bitmap) => ImgRefKind::GRAY16(ImgRef::new(bitmap, self.width, self.height)),
            ImageData::GRAYA8(ref bitmap) => ImgRefKind::GRAYA8(ImgRef::new(bitmap, self.width, self.height)),
            ImageData::GRAYA16(ref bitmap) => ImgRefKind::GRAYA16(ImgRef::new(bitmap, self.width, self.height)),
        }
    }

    /// Returns an enum with `Img<Vec<Pixel>>`
    #[inline]
    pub fn into_imgvec(self) -> ImgVecKind {
        match self.bitmap {
            ImageData::RGB8(bitmap) => ImgVecKind::RGB8(ImgVec::new(bitmap, self.width, self.height)),
            ImageData::RGBA8(bitmap) => ImgVecKind::RGBA8(ImgVec::new(bitmap, self.width, self.height)),
            ImageData::RGB16(bitmap) => ImgVecKind::RGB16(ImgVec::new(bitmap, self.width, self.height)),
            ImageData::RGBA16(bitmap) => ImgVecKind::RGBA16(ImgVec::new(bitmap, self.width, self.height)),
            ImageData::GRAY8(bitmap) => ImgVecKind::GRAY8(ImgVec::new(bitmap, self.width, self.height)),
            ImageData::GRAY16(bitmap) => ImgVecKind::GRAY16(ImgVec::new(bitmap, self.width, self.height)),
            ImageData::GRAYA8(bitmap) => ImgVecKind::GRAYA8(ImgVec::new(bitmap, self.width, self.height)),
            ImageData::GRAYA16(bitmap) => ImgVecKind::GRAYA16(ImgVec::new(bitmap, self.width, self.height)),
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
