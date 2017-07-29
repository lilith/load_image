use image::*;
use lodepng;
use lcms2::*;
use format::*;
use imgref::*;
use convert::*;
use endian::*;
use loader::*;
use alpha::is_opaque;

impl Loader {
    pub fn load_png(&self, data: &[u8]) -> Result<Image, lodepng::Error> {
        let opaque = self.opaque;
        let mut state = lodepng::State::new();
        state.color_convert(false);
        state.read_text_chunks(false);
        state.remember_unknown_chunks(true);

        let (width, height) = state.inspect(data)?;
        if width*height > 10000*10000 {
            return Err(lodepng::Error(92));
        }

        let res = state.decode(data)?;

        let profile = if state.info_png().get("sRGB").is_some() || self.profiles == Profiles::None {
            None
        } else if let Ok(iccp) = state.get_icc() {
            self.process_profile(Profile::new_icc(iccp.as_ref()))
        } else {
            None
        };

        let meta = ImageMeta {
            format: Format::Png, ..ImageMeta::default()
        };

        match res {
            lodepng::Image::RGBA(mut image) => {
                let opaque = opaque || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque, meta))
            },
            lodepng::Image::RGB(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque, meta)),
            lodepng::Image::RGB16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque, meta)),
            lodepng::Image::RGBA16(mut image) => {
                let opaque = opaque || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque, meta))
            },
            lodepng::Image::Grey(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque, meta)),
            lodepng::Image::Grey16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque, meta)),
            lodepng::Image::GreyAlpha(mut image) => {
                let opaque = opaque || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque, meta))
            },
            lodepng::Image::GreyAlpha16(mut image) => {
                let opaque = opaque || is_opaque(image.buffer.as_ref());
                Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque, meta))
            },
            lodepng::Image::RawData(rawdata) => {
                let mut png = state.info_raw_mut();
                let depth = png.bitdepth as u8;
                let pal = match png.colortype() {
                    lodepng::LCT_PALETTE => {
                        let pal = png.palette_mut();
                        let ncolors = pal.len();
                        let opaque = opaque || is_opaque(pal);
                        pal.to_image(profile, 1, ncolors, opaque, meta.clone())
                    },
                    lodepng::LCT_GREY => {
                        let ncolors = 1<<depth;
                        let max = ncolors-1;
                        let mut graypal: Vec<_> = (0..ncolors).map(|c| lodepng::Grey((c*255/max) as u8)).collect();
                        graypal.to_image(profile, 1, ncolors, true, meta.clone())
                    },
                    _ => return Err(lodepng::Error(59))
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
                }.ok_or(lodepng::Error(59))
            },
        }
    }
}

fn from_palette<T: Copy>(buf: &[u8], pal: &[T], bitdepth: u8, width: usize, height: usize) -> Option<ImgVec<T>> {
    match bitdepth {
        8 => Some(ImgVec::new(buf.iter().map(|&c| pal[c as usize]).collect(), width, height)),
        depth @ 1 | depth @ 2 | depth @ 4 => {
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
