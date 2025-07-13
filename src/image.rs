use rgb::RGBA8;
use crate::convert::FromOptions;
use crate::format::Format;
use std::fs;
#[cfg(feature = "stat")]
use std::io;
#[cfg(feature = "stat")]
use std::time;
use rgb::prelude::*;

use crate::export::imgref::{ImgRef, ImgRefKind, ImgVec, ImgVecKind};

pub type ImageMetaChunks = Vec<(ChunkType, Vec<u8>)>;

/// Additional non-image metadata fetched from source files
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkType {
    /// PNG chunks: name + data
    PNG([u8; 4]),
    #[cfg(feature = "mozjpeg")]
    /// App marker
    JPEG(mozjpeg::Marker),
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ImageMeta {
    pub format: Format,
    pub chunks: Vec<(ChunkType, Vec<u8>)>,
    #[cfg(feature = "stat")]
    pub created: u64,
    #[cfg(feature = "stat")]
    pub modified: u64,
}

impl ImageMeta {
    #[cfg(not(feature = "stat"))]
    pub(crate) fn new(format: Format, chunks: ImageMetaChunks, _: Option<fs::Metadata>) -> Self {
        ImageMeta { format, chunks }
    }

    #[cfg(feature = "stat")]
    pub(crate) fn new(format: Format, chunks: ImageMetaChunks, fs_meta: Option<fs::Metadata>) -> Self {
        fn time(t: Result<time::SystemTime, io::Error>) -> Option<u64> {
            t.ok().and_then(|d| d.duration_since(time::UNIX_EPOCH).ok()).map(|d| d.as_secs())
        }
        Self {
            format,
            chunks,
            created: fs_meta.as_ref().and_then(|stat| time(stat.created())).unwrap_or(1638573452),
            modified: fs_meta.and_then(|stat| time(stat.modified())).unwrap_or(1638573452),
        }
    }
}

/// The pixels are in the [`Image::bitmap`] field
///
/// Use [`Image::into_rgba`] if you don't want to deal with multiple pixel formats.
///
/// Pixels will be in sRGB color space.
#[derive(Debug, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub meta: ImageMeta,
    pub bitmap: ImageData,
}

/// Pixels of the image
///
/// The dimensions are in the [`Image`] object owning these. See also [`Image::as_imgref`].
///
/// Call [`Image::into_rgba`] if you don't want to deal with these
///
/// Pixels will be in sRGB color space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageData {
    RGB8(Vec<rgb::RGB8>),
    RGBA8(Vec<rgb::RGBA8>),
    RGB16(Vec<rgb::RGB16>),
    RGBA16(Vec<rgb::RGBA16>),
    GRAY8(Vec<crate::export::rgb::GRAY8>),
    GRAY16(Vec<crate::export::rgb::GRAY16>),
    GRAYA8(Vec<crate::export::rgb::GRAYA8>),
    GRAYA16(Vec<crate::export::rgb::GRAYA16>),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Rotate {
    None,
    FlipX,
    D90,
    D90FlipX,
    D180,
    D180FlipX,
    D270,
    D270FlipX,
}

impl Rotate {
    #[must_use]
    pub const fn from_exif_orientation(orientation: u16) -> Self {
        match orientation {
            2 => Self::FlipX,
            3 => Self::D180,
            4 => Self::D180FlipX,
            5 => Self::D270FlipX,
            6 => Self::D270,
            7 => Self::D90FlipX,
            8 => Self::D90,
            _ => Self::None,
        }
    }
}

impl Image {
    /// Convert pixels to RGBA format. The second returned element is the [`ImageMeta`] object.
    #[must_use]
    pub fn into_rgba(self) -> (ImgVec<rgb::RGBA8>, ImageMeta) {
        let bitmap = match self.bitmap {
            ImageData::RGB8(bitmap) => ImgVec::new(bitmap.into_iter().map(|px| px.with_alpha(255)).collect(), self.width, self.height),
            ImageData::RGBA8(bitmap) => ImgVec::new(bitmap.into_iter().map(|px| px.with_alpha(255)).collect(), self.width, self.height),
            ImageData::RGB16(bitmap) => ImgVec::new(bitmap.into_iter().map(|px| px.map(|c| (c >> 8) as u8).with_alpha(255)).collect(), self.width, self.height),
            ImageData::RGBA16(bitmap) => ImgVec::new(bitmap.into_iter().map(|px| px.map(|c| (c >> 8) as u8)).collect(), self.width, self.height),
            ImageData::GRAY8(bitmap) => ImgVec::new(bitmap.into_iter().map(|g| RGBA8::new(g.value(),g.value(),g.value(),255)).collect(), self.width, self.height),
            ImageData::GRAY16(bitmap) => ImgVec::new(bitmap.into_iter().map(|px| {
                let g = (px.value() >> 8) as u8;
                RGBA8::new(g,g,g,255)
            }).collect(), self.width, self.height),
            ImageData::GRAYA8(bitmap) => ImgVec::new(bitmap.into_iter().map(|g| RGBA8::new(g.v, g.v, g.v, g.a)).collect(), self.width, self.height),
            ImageData::GRAYA16(bitmap) => ImgVec::new(bitmap.into_iter().map(|px| {
                let g = (px.value() >> 8) as u8;
                RGBA8::new(g,g,g, (px.a >> 8) as u8)
            }).collect(), self.width, self.height),
        };
        (bitmap, self.meta)
    }

    /// True if pixel format doesn't support alpha. This function doesn't check pixels.
    #[inline]
    #[must_use]
    pub const fn is_opaque(&self) -> bool {
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

    /// Returns an enum with `Img<&[Pixel]>`, which is like a 2D slice.
    #[inline]
    #[must_use]
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
    #[must_use]
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

    #[must_use]
    pub fn rotated(self, r: Rotate) -> Self {
        let meta = self.meta;
        match self.bitmap {
            ImageData::RGB8(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
            ImageData::RGBA8(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
            ImageData::RGB16(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
            ImageData::RGBA16(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAY8(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAY16(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAYA8(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
            ImageData::GRAYA16(bitmap) => Self::from_opts(Self::rotated_bitmap(ImgVec::new(bitmap, self.width, self.height), r), meta),
        }
    }

    fn rotated_bitmap<T: Copy>(mut bitmap: ImgVec<T>, rotation: Rotate) -> ImgVec<T> {
        let width = bitmap.width();
        let height = bitmap.height();
        let area = width.checked_mul(height).unwrap();
        match rotation {
            Rotate::None => bitmap,
            Rotate::FlipX => {
                bitmap.rows_mut().for_each(|row| row.reverse());
                bitmap
            },
            Rotate::D90 => {
                let mut d = Vec::with_capacity(area);
                for x in (0..width).rev() {
                    for y in 0..height {
                        d.push(bitmap[(x, y)]);
                    }
                }
                ImgVec::new(d, height, width)
            },
            Rotate::D90FlipX => {
                let mut d = Vec::with_capacity(area);
                for x in (0..width).rev() {
                    for y in (0..height).rev() {
                        d.push(bitmap[(x, y)]);
                    }
                }
                ImgVec::new(d, height, width)
            },
            Rotate::D180 => {
                let mut d = Vec::with_capacity(area);
                bitmap.rows().rev().for_each(|row| {
                    d.extend(row.iter().copied().rev());
                });
                ImgVec::new(d, width, height)
            },
            Rotate::D180FlipX => {
                let mut d = Vec::with_capacity(area);
                bitmap.rows().rev().for_each(|row| {
                    d.extend_from_slice(row);
                });
                ImgVec::new(d, width, height)
            },
            Rotate::D270 => {
                let mut d = Vec::with_capacity(area);
                for x in 0..width {
                    for y in (0..height).rev() {
                        d.push(bitmap[(x, y)]);
                    }
                }
                ImgVec::new(d, height, width)
            },
            Rotate::D270FlipX => {
                let mut d = Vec::with_capacity(area);
                for x in 0..width {
                    for y in 0..height {
                        d.push(bitmap[(x, y)]);
                    }
                }
                ImgVec::new(d, height, width)
            },
        }
    }
}

#[cfg(feature = "stat")]
#[test]
fn test_stat() {
    let file = fs::File::open("src/lib.rs").unwrap();
    let m = ImageMeta::new(Format::Unknown, vec![], file.metadata().ok());
    assert!(m.created >= 1499358961);
    assert!(m.modified >= 1499358961);
    assert!(m.modified >= m.created);
}
