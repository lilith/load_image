use std::panic::catch_unwind;
use rexif::{ExifTag, TagValue};

const ADOBE98_CHROMATICITIES: &[f64] = &[0.64, 0.33, 0.21, 0.71, 0.15, 0.06];

pub(crate) fn parse_exif(data: &[u8]) -> (u16, bool) {

    // panic from drop is not an issue
    let exif = catch_unwind(|| {
        rexif::parse_buffer(data)
    });

    let mut orientation = 1;
    let mut is_adobe_1998 = false;
    if let Ok(Ok(parsed)) = exif {
        for f in parsed.entries {
            match (f.tag, f.value) {
                (ExifTag::PrimaryChromaticities, TagValue::URational(n)) => {
                    if n.len() == ADOBE98_CHROMATICITIES.len() &&
                        n.iter().zip(ADOBE98_CHROMATICITIES).all(|(r,a)| (r.value()-a).abs() < 0.001) {
                        is_adobe_1998 = true;
                    }
                },
                (ExifTag::Orientation, TagValue::U16(n)) => {
                    if let Some(&n) = n.get(0) {
                        orientation = n;
                    }
                },
                _ => {},
            };
        }
    }
    (orientation, is_adobe_1998)
}
