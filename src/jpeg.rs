use lodepng;
use lcms2::*;
use rgb::*;
use image::*;
use format::*;
use pixel_format::*;
use convert::*;
use std::panic;
use mozjpeg::{Decompress, Marker};
use mozjpeg::ColorSpace::*;
use exif;

fn get_profile(dinfo: &Decompress) -> Option<Profile> {
    let mut profile_markers = Vec::new();

    for m in dinfo.markers() {
        let data = m.data;
        if m.marker == Marker::APP(2) && data.len() > 14 &&
            data[12] <= data[13] &&
            b"ICC_PROFILE\0" == &data[0..12] {
            profile_markers.push(data);
        }
    }

    let profile = if !profile_markers.is_empty() {
        profile_markers.sort_by_key(|data| data[12]);
        let mut icc = Vec::new();
        for data in profile_markers {
            icc.extend(&data[14..]);
        }
        Profile::new_icc(&icc[..]).ok()
    } else {
        None
    };

    profile
}

fn get_orientation(dinfo: &Decompress) -> u16 {
    for m in dinfo.markers() {
        let data = m.data;
        if m.marker == Marker::APP(1) && data.len() > 8 &&
            &data[0..6] == b"Exif\0\0" {
            if let Ok((exif_fields, _)) = exif::parse_exif(&data[6..]) {
                for f in exif_fields {
                    if f.tag == exif::tag::Orientation {
                        if let exif::Value::Short(n) = f.value {
                            if let Some(&n) = n.get(0) {return n;}
                        }
                    }
                }
            }
        }
    }
    return 1;
}

pub fn load_jpeg(data: &[u8]) -> Result<Image, lodepng::Error> {
    let thread_res = panic::catch_unwind(|| {
        let mut dinfo = Decompress::new();
        dinfo.set_mem_src(data);
        dinfo.save_marker(Marker::APP(1)); // Exif
        dinfo.save_marker(Marker::APP(2)); // Profile
        assert!(dinfo.read_header(true));
        assert!(dinfo.start_decompress());
        let width = dinfo.output_width();
        let height = dinfo.output_height();

        if width*height > 10000*10000 {
            return Err(lodepng::Error(92));
        }

        let profile = get_profile(&dinfo);
        let orientation = get_orientation(&dinfo);

        let img = match dinfo.out_color_space() {
            JCS_RGB => {
                let mut rgb: Vec<RGB8> = dinfo.read_scanlines().unwrap();
                rgb.to_image(profile, width, height, true, Format::Jpeg)
            },
            JCS_CMYK => {
                let mut rgb: Vec<CMYK> = dinfo.read_scanlines().unwrap();
                rgb.to_image(profile, width, height, true, Format::Jpeg)
            },
            JCS_GRAYSCALE => {
                let mut g: Vec<GRAY8> = dinfo.read_scanlines().unwrap();
                g.to_image(profile, width, height, true, Format::Jpeg)
            },
            _ => return Err(lodepng::Error(59))
        };
        Ok((img, orientation))
    });

    if thread_res.is_err() {
        return Err(lodepng::Error(28));
    }
    let (img, orientation) = thread_res.unwrap()?;

    Ok(match orientation {
        1 | 0 => img,
        2 => img.rotated(Rotate::FlipX),
        3 => img.rotated(Rotate::D180),
        4 => img.rotated(Rotate::D180FlipX),
        5 => img.rotated(Rotate::D270FlipX),
        6 => img.rotated(Rotate::D270),
        7 => img.rotated(Rotate::D90FlipX),
        8 => img.rotated(Rotate::D90),
        x => panic!("unsupported rotation {}", x),
    })
}
