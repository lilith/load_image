use crate::Format;
use crate::FromOptions;
use crate::Image;
use crate::ImageMeta;
use crate::Loader;
use std::fs;
use libwebp::error::WebPSimpleError;
use imgref::ImgVec;

impl Loader {
    pub(crate) fn load_webp(&self, data: &[u8], fs_meta: Option<fs::Metadata>) -> Result<Image, WebPSimpleError> {
        let opts = ImageMeta::new(Format::WebP, fs_meta);
        if self.opaque {
            let (w, h, pixels) = libwebp::WebPDecodeRGB(data)?;
            Ok(Image::from_opts(ImgVec::new(pixels.to_vec(), w as _, h as _), opts))
        } else {
            let (w, h, pixels) = libwebp::WebPDecodeRGBA(data)?;
            Ok(Image::from_opts(ImgVec::new(pixels.to_vec(), w as _, h as _), opts))
        }
    }
}

#[test]
fn poke_webp_test() {
    let a = crate::load_image("tests/img/test.webp", false).unwrap();
    assert_eq!(20, a.width);
    assert_eq!(20, a.height);
}
