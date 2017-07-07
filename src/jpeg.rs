use lodepng;
use lcms2::*;
use rgb::*;
use image::*;
use format::*;
use pixel_format::*;
use convert::*;
use loader::*;
use profiles;
use std::panic;
use mozjpeg::{Decompress, Marker};
use mozjpeg::ColorSpace::*;
use rexif;

const ADOBE98_CHROMATICITIES: &'static [f64] = &[0.64, 0.33, 0.21, 0.71, 0.15, 0.06];

impl Loader {
    fn get_jpeg_profile(&self, dinfo: &Decompress) -> Option<Profile> {
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
            self.process_profile(Profile::new_icc(&icc[..]))
        } else {
            None
        };

        profile
    }

    fn get_exif_data(dinfo: &Decompress) -> (u16, bool) {
        let mut orientation = 1;
        let mut is_adobe_1998 = false;

        for m in dinfo.markers() {
            let data = m.data;
            if m.marker != Marker::APP(1) || data.len() < 12 || &data[0..6] != b"Exif\0\0" {
                continue;
            }
            if let Ok(parsed) = rexif::parse_buffer(&data[6..]) {
                for f in parsed.entries {
                    match (f.tag, f.value) {
                        (rexif::ExifTag::PrimaryChromaticities, rexif::TagValue::URational(n)) => {
                            if n.len() == ADOBE98_CHROMATICITIES.len() &&
                                n.iter().zip(ADOBE98_CHROMATICITIES).all(|(r,a)| (r.value()-a).abs() < 0.001) {
                                is_adobe_1998 = true;
                            }
                        },
                        (rexif::ExifTag::Orientation, rexif::TagValue::U16(n)) => {
                            if let Some(&n) = n.get(0) {
                                orientation = n;
                            }
                        },
                        _ => {},
                    };
                }
            }
        }
        return (orientation, is_adobe_1998);
    }

    pub fn load_jpeg(&self, data: &[u8]) -> Result<Image, lodepng::Error> {
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

            let (orientation, is_adobe_1998) = Self::get_exif_data(&dinfo);
            let profile = if self.profiles == Profiles::None {
                None
            } else if let Some(embedded) = self.get_jpeg_profile(&dinfo) {
                Some(embedded)
            } else if is_adobe_1998 {
                Profile::new_icc(profiles::ADOBE1998).ok()
            } else {
                None
            };

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
            2 => img.rotated(Rotate::FlipX),
            3 => img.rotated(Rotate::D180),
            4 => img.rotated(Rotate::D180FlipX),
            5 => img.rotated(Rotate::D270FlipX),
            6 => img.rotated(Rotate::D270),
            7 => img.rotated(Rotate::D90FlipX),
            8 => img.rotated(Rotate::D90),
            _ => img, // 1 is expected, but in practice there's lots of images with bogus rotation
        })
    }
}
