use mozjpeg;
use lodepng;
use lcms2::*;
use rgb::*;
use image::*;
use pixel_format::*;
use convert::*;

pub fn load_jpeg(data: &[u8]) -> Result<Image, lodepng::Error> {
    let mut dinfo = mozjpeg::Decompress::new();
    dinfo.set_mem_src(data);
    dinfo.save_marker(mozjpeg::Marker::APP(2));
    assert!(dinfo.read_header(true));
    assert!(dinfo.start_decompress());
    let width = dinfo.output_width();
    let height = dinfo.output_height();

    let profile = if let Some(marker) = dinfo.markers().next() {
        let data = marker.data;
        if b"ICC_PROFILE\0" == &data[0..12] {
            let icc = &data[14..];
            Profile::new_icc(icc).ok()
        } else {None}
    } else {None};

    match dinfo.out_color_space() {
        mozjpeg::ColorSpace::JCS_RGB => {
            let mut rgb: Vec<RGB8> = dinfo.read_scanlines().unwrap();
            Ok(rgb.to_image(profile, width, height, true))
        },
        mozjpeg::ColorSpace::JCS_CMYK => {
            let mut rgb: Vec<CMYK> = dinfo.read_scanlines().unwrap();
            Ok(rgb.to_image(profile, width, height, true))
        },
        mozjpeg::ColorSpace::JCS_GRAYSCALE => {
            let mut g: Vec<GRAY8> = dinfo.read_scanlines().unwrap();
            Ok(g.to_image(profile, width, height, true))
        },
        _ => Err(lodepng::Error(59)),
    }
}
