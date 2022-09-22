use bytemuck::cast_slice;
use lodepng::Grey;

use crate::convert::*;
use crate::exif::parse_exif;
use crate::format::*;
use crate::image::*;
use crate::loader::*;
use crate::pixel_format::*;
use jpeg_decoder::PixelFormat;
use lcms2::*;
use rgb::*;
use std::fs;

impl Loader {
    pub(crate) fn load_jpeg(&self, mut data: &[u8], fs_meta: Option<fs::Metadata>) -> Result<Image, crate::Error> {
        let mut dec = jpeg_decoder::Decoder::new(&mut data);
        let mut pixels = dec.decode()?;
        let info = dec.info().ok_or(crate::Error::UnsupportedJpeg)?;
        let exif = dec.exif_data();

        let (orientation, is_adobe_1998) = exif.map(parse_exif).unwrap_or((1, false));

        let profile = dec.icc_profile().as_deref()
            .or_else(|| if is_adobe_1998 { Some(crate::profiles::ADOBE1998) } else { None })
            .and_then(|icc| self.process_profile(Profile::new_icc(&icc)));

        let width = info.width.into();
        let height = info.height.into();
        // FIXME: metadata not preserved
        let meta = ImageMeta::new(Format::Jpeg, vec![], fs_meta);
        let img = match info.pixel_format {
            PixelFormat::L8 => pixels.as_gray_mut().to_image(profile, width, height, true, meta),
            PixelFormat::L16 => {
                let (unaligned, slice_u16, _) = unsafe { pixels.align_to::<u16>() };
                assert!(unaligned.is_empty(), "https://github.com/image-rs/jpeg-decoder/issues/223");
                slice_u16.iter().copied().map(Grey::new).collect::<Vec<_>>().to_image(profile, width, height, true, meta)
            }
            PixelFormat::RGB24 => pixels.as_rgb_mut().to_image(profile, width, height, true, meta),
            PixelFormat::CMYK32 => cast_slice::<u8, CMYK>(&pixels).to_image(profile, width, height, true, meta),
        };

        Ok(img.rotated(Rotate::from_exif_orientation(orientation)))
    }
}
