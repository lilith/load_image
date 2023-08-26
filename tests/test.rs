use load_image::export::imgref::*;
use load_image::export::rgb::*;
use load_image::*;

fn tou16(v: u8) -> u16 {
    let v = v as u16;
    (v << 8) | v
}

#[cfg(test)]
fn convert(img: &Image) -> ImgVec<RGBA16> {
    match img.bitmap {
        ImageData::RGB8(ref bitmap) => ImgVec::new(bitmap.iter().map(|c|c.map(tou16).alpha(65535)).collect(), img.width, img.height),
        ImageData::RGBA8(ref bitmap) => ImgVec::new(bitmap.iter().map(|c|c.map(tou16)).collect(), img.width, img.height),
        ImageData::RGB16(ref bitmap) => ImgVec::new(bitmap.iter().map(|c|c.alpha(65535)).collect(), img.width, img.height),
        ImageData::RGBA16(ref bitmap) => ImgVec::new(bitmap.clone(), img.width, img.height),
        ImageData::GRAY8(ref bitmap) => ImgVec::new(bitmap.iter().map(|c|{
            let c = tou16(c.0);
            RGBA::new(c,c,c,65535)
        }).collect(), img.width, img.height),
        ImageData::GRAYA8(ref bitmap) => ImgVec::new(bitmap.iter().map(|p|{
            let c = tou16(p.0);
            RGBA::new(c,c,c,tou16(p.1))
        }).collect(), img.width, img.height),
        ImageData::GRAY16(ref bitmap) => ImgVec::new(bitmap.iter().map(|c|RGBA::new(c.0,c.0,c.0,65535)).collect(), img.width, img.height),
        ImageData::GRAYA16(ref bitmap) => ImgVec::new(bitmap.iter().map(|p|{RGBA::new(p.0,p.0,p.0,p.1)}).collect(), img.width, img.height),
    }
}

#[cfg(test)]
fn compare(left: &Image, right: &Image) -> f64 {
    let left = convert(left);
    let right = convert(right);
    assert_eq!(left.width(), right.width());
    assert_eq!(left.height(), right.height());
    let ppx = left.pixels()
        .zip(right.pixels())
        .map(|(a, b)| {
            let a = a.map(|c|c as i64);
            let b = b.map(|c|c as i64);
            let d = RGBA{
                r: (a.r*a.a - b.r*b.a),
                g: (a.g*a.a - b.g*b.a),
                b: (a.b*a.a - b.b*b.a),
                a: (a.a*65536 - b.a*65536),
            };
            (d.r.saturating_mul(d.r).saturating_add(d.g.saturating_mul(d.g)).saturating_add(d.b.saturating_mul(d.b)).saturating_add(d.a.saturating_mul(d.a))) as u64 >> 16
        })
        .sum::<u64>() / ((left.width() as u64 * left.height() as u64) << 24);
    ppx as f64 / (1<<24) as f64
}

#[test]
#[cfg(feature = "mozjpeg")]
#[allow(deprecated)]
fn image_gray() {
    let g0 = load_path("tests/img/gray1-rgba16.png").unwrap();
    let g1 = load_path("tests/img/gray1-rgba.png").unwrap();
    let g2 = load_path("tests/img/gray1-pal.png").unwrap();
    let g3 = load_path("tests/img/gray1-gray.png").unwrap();
    let g4 = load_path("tests/img/gray1.jpg").unwrap();
    let g5 = load_image("tests/img/gray1-rgba.png", true).unwrap();

    assert!(g0.is_opaque());
    assert!(g3.is_opaque());
    assert!(g4.is_opaque());

    let diff = compare(&g0, &g1);
    assert!(diff < 0.00001, "{}", diff);

    let diff = compare(&g1, &g2);
    assert!(diff < 0.00001, "{}", diff);

    let diff = compare(&g1, &g3);
    assert!(diff < 0.00001, "{}", diff);

    let diff = compare(&g1, &g4);
    assert!(diff < 0.00006, "{}", diff);

    let diff = compare(&g4, &g5);
    assert!(diff < 0.00006, "{}", diff);

    let diff = compare(&g1, &g5);
    assert!(diff < 0.00001, "{}", diff);

    match (g1.bitmap, g5.bitmap) {
        (ImageData::RGBA16(_), ImageData::RGB16(_)) => {},
        (ImageData::RGBA8(_), ImageData::RGBA8(_)) => {}, // Case when profiles are skipped
        _ => panic!("opaque flag is supposed to return non-alpha type"),
    }
}

#[test]
#[cfg(feature = "mozjpeg")]
#[allow(deprecated)]
fn image_implied1998_profile() {
    let adobe98 = load_image("tests/img/adobe1998exif.jpg", true).unwrap();
    let expected = load_image("tests/img/adobe1998assrgb.jpg", true).unwrap();

    let diff = compare(&adobe98, &expected);
    assert!(diff < 0.0008, "98 {}", diff);
}

#[test]
#[cfg(feature = "mozjpeg")]
#[allow(deprecated)]
fn image_gray_profile() {
    let gp1 = load_path("tests/img/gray-profile.png").expect("tests/img/gray-profile.png");
    let gp1o = load_image("tests/img/gray-profile.png", true).expect("tests/img/gray-profile.png");
    let gp2 = load_path("tests/img/gray-profile2.png").expect("tests/img/gray-profile2.png");
    let gp3 = load_path("tests/img/gray-profile.jpg").expect("tests/img/gray-profile.jpg");

    let diff = compare(&gp1, &gp2);
    assert!(diff < 0.0003, "{}", diff);

    let diff = compare(&gp1, &gp3);
    assert!(diff < 0.0003, "{}", diff);

    match gp1o.bitmap {
        ImageData::GRAY16(_) => {},
        _ => panic!("opaque flag"),
    };

    match gp1.bitmap {
        ImageData::GRAY16(_) => {},
        _ => panic!("opaque auto-detect"),
    };
}

#[test]
#[cfg(feature = "mozjpeg")]
fn image_load1() {
    let prof_jpg = load_path("tests/img/profile.jpg").unwrap();
    let prof_png = load_path("tests/img/profile.png").unwrap();
    let diff = compare(&prof_jpg, &prof_png);
    assert!(diff <= 0.002);

    let strip_jpg = load_path("tests/img/profile-stripped.jpg").unwrap();
    let diff = compare(&strip_jpg, &prof_jpg);
    assert!(diff > 0.002, "{}", diff);

    let strip_png = load_path("tests/img/profile-stripped.png").unwrap();
    let diff = compare(&strip_jpg, &strip_png);
    assert!(diff > 0.002, "{}", diff);
}

#[test]
#[cfg(feature = "mozjpeg")]
fn image_load_no_profiles() {
    let prof_jpg = Loader::new().profiles(Profiles::None).load_path("tests/img/profile.jpg").unwrap();

    let strip_jpg = load_path("tests/img/profile-stripped.jpg").unwrap();
    let diff = compare(&strip_jpg, &prof_jpg);
    assert!(diff < 0.002, "{} jpg", diff);

    match prof_jpg.bitmap {
        ImageData::RGB8(_) => {},
        _ => panic!("Expected plain RGB"),
    }
}

#[test]
fn image_load_all_profiles() {
    let prof_png = Loader::new().profiles(Profiles::All).load_path("tests/img/profile.png").unwrap();

    match prof_png.bitmap {
        ImageData::RGB16(_) => {},
        _ => panic!("Expected widening due to profile"),
    }
}

#[test]
#[cfg(feature = "mozjpeg")]
fn image_load_some_profiles() {
    let prof_png = Loader::new().profiles(Profiles::NonsRGB).load_path("tests/img/profile.png").unwrap();
    assert!(prof_png.is_opaque());
    match prof_png.bitmap {
        ImageData::RGB8(_) => {},
        _ => panic!("Expected no widening due to skipped sRGB embedded profile"),
    }

    let prof_jpg = Loader::new().profiles(Profiles::NonsRGB).load_path("tests/img/profile.jpg").unwrap();
    match prof_jpg.bitmap {
        ImageData::RGB16(_) => {},
        _ => panic!("Expected widening due to profile"),
    }
}

#[test]
fn image_4bit() {
    let im1 = load_path("tests/img/tile1.png").unwrap();
    let im2 = load_path("tests/img/tile2.png").unwrap();
    assert!(!im1.is_opaque());
    assert!(!im2.is_opaque());
    let diff = compare(&im1, &im2);
    assert!(diff <= 0.00002);
}

#[test]
#[cfg(feature = "mozjpeg")]
#[allow(deprecated)]
fn image_cmyk() {
    let im1 = load_image("tests/img/cmyk.png", true).unwrap();
    let im2 = load_image("tests/img/cmyk.jpg", true).unwrap();
    assert!(im1.is_opaque());
    assert!(im2.is_opaque());

    let diff = compare(&im1, &im2);
    assert!(diff <= 0.002);
    assert_eq!(Format::Jpeg, im2.meta.format);
}

#[test]
fn image_bw() {
    let res = load_path("tests/img/1bit.png").unwrap();
    assert_eq!(Format::Png, res.meta.format);
    assert!(res.is_opaque());
}

#[test]
fn pngtestsuite() {
    for entry in std::fs::read_dir("tests/pngtestsuite").unwrap().filter_map(|p| p.ok()) {
        let path = entry.path();
        let filenamestr = path.file_name().unwrap().to_string_lossy();
        // ignore signature test, since fallback for jpeg panics
        // ignore checksum check, since lodepng doesn't do it
        if !filenamestr.ends_with(".png") || filenamestr.starts_with("xs") || filenamestr.starts_with("xcs") {
            continue;
        }
        let res = load_path(&path);
        if filenamestr.starts_with('x') {
            assert!(res.is_err());
        } else {
            let res = res.unwrap();
            assert_eq!(Format::Png, res.meta.format);
        }
    }
}

#[test]
#[cfg(feature = "mozjpeg")]
#[allow(deprecated)]
fn exif_test() {
    for orient in &["top-left", "top-right", "bottom-left", "bottom-right", "left-bottom", "left-top", "right-bottom", "right-top"] {
        let png_path = format!("tests/img/exif-{orient}.png");
        let expected = load_image(&png_path, true).expect(&png_path);
        let jpeg_path = format!("tests/img/exif-{orient}.jpg");
        let actual = load_image(&jpeg_path, true).expect(&jpeg_path);
        let diff = compare(&expected, &actual);

        assert!(actual.is_opaque());

        assert!(diff <= 0.0002, "orient {orient} = {diff}");
        assert_eq!(Format::Jpeg, actual.meta.format);
        assert_eq!(Format::Png, expected.meta.format);
    }
}

#[test]
fn nonsense() {
    assert!(load_data(&[0u8]).is_err());
    assert!(load_data(&vec![0xFEu8; 1000]).is_err());
    assert!(load_data(&vec![0xFFu8; 1000]).is_err());
}
