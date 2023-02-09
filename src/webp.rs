use crate::Format;
use crate::FromOptions;
use crate::Image;
use crate::ImageMeta;
use crate::Loader;
use imgref::ImgVec;
use rgb::FromSlice;
use std::fs;

impl Loader {
    pub(crate) fn load_webp(&self, data: &[u8], fs_meta: Option<fs::Metadata>) -> Result<Image, crate::Error> {
        let opts = ImageMeta::new(Format::WebP, vec![], fs_meta);
        if self.discard_alpha {
            let (w, h, pixels) = libwebp::WebPDecodeRGB(data).map_err(|_| crate::Error::WebP)?;
            let w = w as usize;
            let h = h as usize;
            self.check_dimensions(w, h)?;
            Ok(Image::from_opts(ImgVec::new(pixels.as_rgb().to_vec(), w, h), opts))
        } else {
            let (w, h, pixels) = libwebp::WebPDecodeRGBA(data).map_err(|_| crate::Error::WebP)?;
            let w = w as usize;
            let h = h as usize;
            self.check_dimensions(w, h)?;
            Ok(Image::from_opts(ImgVec::new(pixels.as_rgba().to_vec(), w, h), opts))
        }
    }
}

#[test]
fn poke_webp_test() {
    let a = crate::load_path("tests/img/test.webp").unwrap();
    assert_eq!(20, a.width);
    assert_eq!(20, a.height);
}
