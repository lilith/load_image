use crate::alpha::is_opaque;
use crate::convert::*;
use crate::endian::*;
use crate::format::*;
use crate::image::*;
use crate::loader::*;
use imgref::ImgVec;
use lcms2::Profile;

use lodepng::ChunkPosition;
use rgb::Gray;
use std::fs;

impl Loader {
    pub(crate) fn load_png(&self, data: &[u8], fs_meta: Option<fs::Metadata>) -> Result<Image, crate::Error> {
        let discard_alpha = self.discard_alpha;
        let mut state = lodepng::Decoder::new();
        state.color_convert(false);
        state.read_text_chunks(false);
        state.remember_unknown_chunks(true); // always true, because ICC

        let (width, height) = state.inspect(data)?;
        self.check_dimensions(width, height)?;

        let res = state.decode(data)?;

        let profile = if state.info_png().get("sRGB").is_some() || self.profiles == Profiles::None {
            None
        } else if let Ok(iccp) = state.get_icc() {
            self.process_profile(Profile::new_icc(iccp.as_ref()))
        } else {
            None
        };

        let chunks = [ChunkPosition::IHDR, ChunkPosition::PLTE, ChunkPosition::IDAT].iter()
            .flat_map(|&pos| state.info_png().try_unknown_chunks(pos)
            .filter_map(|ch| ch.ok())
            .map(|ch| {
                (ChunkType::PNG(ch.name()), ch.data().to_vec())
            })).collect();

        let meta = ImageMeta::new(Format::Png, chunks, fs_meta);

        match res {
            lodepng::Image::RGBA(mut image) => {
                let discard_alpha = discard_alpha || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.to_image(profile, image.width, image.height, discard_alpha, meta))
            },
            lodepng::Image::RGB(mut image) => Ok(image.buffer.to_image(profile, image.width, image.height, discard_alpha, meta)),
            lodepng::Image::RGB16(mut image) => Ok(image.buffer.to_native().to_image(profile, image.width, image.height, discard_alpha, meta)),
            lodepng::Image::RGBA16(mut image) => {
                let discard_alpha = discard_alpha || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.to_native().to_image(profile, image.width, image.height, discard_alpha, meta))
            },
            lodepng::Image::Grey(mut image) => Ok(image.buffer.to_image(profile, image.width, image.height, discard_alpha, meta)),
            lodepng::Image::Grey16(mut image) => Ok(image.buffer.to_native().to_image(profile, image.width, image.height, discard_alpha, meta)),
            lodepng::Image::GreyAlpha(mut image) => {
                let discard_alpha = discard_alpha || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.to_image(profile, image.width, image.height, discard_alpha, meta))
            },
            lodepng::Image::GreyAlpha16(mut image) => {
                let discard_alpha = discard_alpha || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.to_native().to_image(profile, image.width, image.height, discard_alpha, meta))
            },
            lodepng::Image::RawData(rawdata) => {
                let png = state.info_raw_mut();
                let depth = png.bitdepth() as u8;
                let pal = match png.colortype() {
                    lodepng::ColorType::PALETTE => {
                        let pal = png.palette_mut();
                        let ncolors = pal.len();
                        let discard_alpha = discard_alpha || is_opaque(pal);
                        pal.to_image(profile, 1, ncolors, discard_alpha, meta.clone())
                    },
                    lodepng::ColorType::GREY => {
                        let ncolors = 1 << depth;
                        let max = ncolors - 1;
                        let mut graypal: Vec<_> = (0..ncolors).map(|c| Gray((c * 255 / max) as u8)).collect();
                        graypal.to_image(profile, 1, ncolors, true, meta.clone())
                    },
                    _ => return Err(lodepng::Error::new(59).into()),
                };
                match pal.bitmap {
                    ImageData::RGB8(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                    ImageData::RGBA8(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                    ImageData::RGB16(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                    ImageData::RGBA16(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                    ImageData::GRAY8(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                    ImageData::GRAYA8(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                    ImageData::GRAY16(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                    ImageData::GRAYA16(ref pal) => from_palette(rawdata.buffer.as_ref(), pal, depth, rawdata.width, rawdata.height).map(|i|Image::from_opts(i, meta)),
                }.ok_or(lodepng::Error::new(59).into())
            },
        }
    }
}

fn from_palette<T: Copy>(buf: &[u8], pal: &[T], bitdepth: u8, width: usize, height: usize) -> Option<ImgVec<T>> {
    match bitdepth {
        8 => Some(ImgVec::new(buf.iter().map(|&c| pal[c as usize]).collect(), width, height)),
        depth @ (1 | 2 | 4) => {
            let px_per_byte = 8 / depth;
            let mask = (1<<depth) - 1;
            Some(ImgVec::new(buf.iter()
                                 .flat_map(|c| (0..px_per_byte).rev().map(move |n| pal[(c >> (n * depth) & mask) as usize]))
                                 .take(width * height)
                                 .collect(),
                             width,
                             height))
        },
        _ => None,
    }
}
