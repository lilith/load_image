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

    let profile = {
        let mut profile_markers = Vec::new();
        for m in dinfo.markers() {
            let data = m.data;
            if m.marker == mozjpeg::Marker::APP(2) &&
                data.len() > 14 && data[12] <= data[13] &&
                b"ICC_PROFILE\0" == &data[0..12] {
                profile_markers.push(data);
            }
        }
        if profile_markers.is_empty() {
           None
        } else {
            profile_markers.sort_by_key(|data| data[12]);
            let mut icc = Vec::new();
            for data in profile_markers {
                icc.extend(&data[14..]);
            }
            Profile::new_icc(&icc[..]).ok()
        }
    };

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
