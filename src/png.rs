use image::*;
use lodepng;
use lcms2::*;
use bitmap::*;
use convert::*;
use endian::*;

pub fn load_png(data: &[u8], opaque: bool) -> Result<Image, lodepng::Error> {
    let mut state = lodepng::State::new();
    state.color_convert(false);
    state.read_text_chunks(false);
    state.remember_unknown_chunks(true);
    let res = state.decode(data)?;

    let profile = if state.info_png().get("sRGB").is_some() {
        None
    } else if let Ok(iccp) = state.get_icc() {
        Profile::new_icc(iccp.as_ref()).ok()
    } else {
        None
    };

    match res {
        lodepng::Image::RGBA(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RGB(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RGB16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RGBA16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::Grey(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::Grey16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::GreyAlpha(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::GreyAlpha16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RawData(rawdata) => {
            let mut png = state.info_raw_mut();
            let depth = png.bitdepth as u8;
            let pal = match png.colortype() {
                lodepng::LCT_PALETTE => {
                    let pal = png.palette_mut();
                    let ncolors = pal.len();
                    pal.to_image(profile, 1, ncolors, opaque)
                },
                lodepng::LCT_GREY => {
                    let ncolors = 1<<depth;
                    let max = ncolors-1;
                    let mut graypal: Vec<_> = (0..ncolors).map(|c| lodepng::Grey((c*255/max) as u8)).collect();
                    graypal.to_image(profile, 1, ncolors, opaque)
                },
                _ => return Err(lodepng::Error(59))
            };
            match pal {
                Image::RGB8(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGB8).ok_or(lodepng::Error(59)),
                Image::RGBA8(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGBA8).ok_or(lodepng::Error(59)),
                Image::RGB16(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGB16).ok_or(lodepng::Error(59)),
                Image::RGBA16(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGBA16).ok_or(lodepng::Error(59)),
                Image::GRAY8(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::GRAY8).ok_or(lodepng::Error(59)),
                Image::GRAYA8(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::GRAYA8).ok_or(lodepng::Error(59)),
                Image::GRAY16(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::GRAY16).ok_or(lodepng::Error(59)),
                Image::GRAYA16(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::GRAYA16).ok_or(lodepng::Error(59)),
            }
        },
    }
}


fn from_palette<T: Copy>(buf: &[u8], pal: &[T], bitdepth: u8, width: usize, height: usize) -> Option<Bitmap<T>> {
    match bitdepth {
        8 => Some(Bitmap::new(buf.iter().map(|&c| pal[c as usize]).collect(), width, height)),
        depth @ 1 | depth @ 2 | depth @ 4 => {
            let px_per_byte = 8 / depth;
            let mask = (1<<depth) - 1;
            Some(Bitmap::new(buf.iter()
                                 .flat_map(|c| (0..px_per_byte).rev().map(move |n| pal[(c >> (n * depth) & mask) as usize]))
                                 .take(width * height)
                                 .collect(),
                             width,
                             height))
        },
        _ => None,
    }
}
