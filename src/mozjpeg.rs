use crate::convert::*;
use crate::exif::*;
use crate::format::*;
use crate::image::*;
use crate::loader::*;
use crate::pixel_format::CMYK;
use crate::profiles;
use lcms2::Profile;
use mozjpeg::{Decompress, Marker, ALL_MARKERS};
use rgb::*;
use std::fs;
use std::panic;

impl Loader {
    fn get_jpeg_profile<R>(&self, dinfo: &Decompress<R>) -> Option<Profile> {
        let mut profile_markers = Vec::new();

        for m in dinfo.markers() {
            let data = m.data;
            if m.marker == Marker::APP(2) && data.len() > 14 &&
                data[12] <= data[13] &&
                b"ICC_PROFILE\0" == &data[0..12] {
                profile_markers.push(data);
            }
        }

        if !profile_markers.is_empty() {
            profile_markers.sort_by_key(|data| data[12]);
            let mut icc = Vec::new();
            for data in profile_markers {
                icc.extend(&data[14..]);
            }
            self.process_profile(Profile::new_icc(&icc[..]))
        } else {
            None
        }
    }

    fn get_exif_data<R>(dinfo: &Decompress<R>) -> (u16, bool) {
        for m in dinfo.markers() {
            let data = m.data;
            if m.marker != Marker::APP(1) || data.len() < 12 || &data[0..6] != b"Exif\0\0" {
                continue;
            }
            return parse_exif(&data[6..]);
        }
        (1, false)
    }

    pub(crate) fn load_mozjpeg(&self, data: &[u8], fs_meta: Option<fs::Metadata>) -> Result<Image, crate::Error> {
        let thread_res = panic::catch_unwind(move || {
            let which_markers = if self.metadata {
                ALL_MARKERS
            } else {
                &[
                    Marker::APP(1), /* Exif */
                    Marker::APP(2), /* Profile */
                ]
            };
            let dinfo = Decompress::with_markers(which_markers).from_mem(data)?;
            let width = dinfo.width();
            let height = dinfo.height();

            if width * height > 10000 * 10000 {
                return Err(crate::Error::ImageTooLarge);
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
            let chunks = dinfo.markers().map(|m| {
                (ChunkType::JPEG(m.marker), m.data.to_vec())
            }).collect();
            let meta = ImageMeta::new(Format::Jpeg, chunks, fs_meta);

            let img = match dinfo.image()? {
                mozjpeg::Format::RGB(mut dinfo) => {
                    let mut rgb: Vec<RGB8> = dinfo.read_scanlines()?;
                    rgb.to_image(profile, width, height, true, meta)
                },
                mozjpeg::Format::CMYK(mut dinfo) => {
                    let cmyk: Vec<CMYK> = dinfo.read_scanlines()?;
                    cmyk.as_slice().to_image(profile, width, height, true, meta)
                },
                mozjpeg::Format::Gray(mut dinfo) => {
                    let mut g: Vec<rgb::alt::Gray<u8>> = dinfo.read_scanlines()?;
                    g.to_image(profile, width, height, true, meta)
                },
            };
            Ok((img, orientation))
        });

        let (img, orientation) = thread_res.map_err(|e| {
            let string = e.downcast::<String>().map(|e| *e)
                .or_else(|e| e.downcast::<&'static str>().map(|s| String::from(*s)));
            if let Ok(e) = string {
                crate::Error::Jpeg(e)
            } else {
                crate::Error::UnsupportedJpeg
            }
        })??;

        Ok(img.rotated(Rotate::from_exif_orientation(orientation)))
    }
}
