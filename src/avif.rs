use crate::Format;
use crate::FromOptions;
use crate::Image;
use crate::ImageMeta;
use crate::Loader;
use aom_decode::avif;
use aom_decode::Config;
use std::fs;

impl Loader {
    pub(crate) fn load_avif(&self, data: &[u8], fs_meta: Option<fs::Metadata>) -> Result<Image, aom_decode::Error> {
        let mut d = avif::Avif::decode(data, &Config {
            threads: num_cpus::get(),
        })?;

        let meta = ImageMeta::new(Format::Avif, fs_meta);
        Ok(match d.convert()? {
            avif::Image::RGB8(img) => Image::from_opts(img, meta),
            avif::Image::RGBA8(img) => Image::from_opts(img, meta),
            avif::Image::RGB16(img) => Image::from_opts(img, meta),
            avif::Image::RGBA16(img) => Image::from_opts(img, meta),
            avif::Image::Gray8(img) => Image::from_opts(img, meta),
            avif::Image::Gray16(img) => Image::from_opts(img, meta),
        })
    }
}

#[test]
fn poke_avif_test() {
    let a = crate::load_image("tests/img/test.avif", false).unwrap();
    assert_eq!(18, a.width);
    assert_eq!(6, a.height);
}
