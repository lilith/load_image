#![allow(unknown_lints)]

extern crate lodepng;
extern crate lcms2;
extern crate mozjpeg;
extern crate file;
extern crate rgb;

mod pixel_format;
mod bitmap;
mod endian;
mod png;
mod image;
mod convert;

use std::io::Read;
use std::path::Path;
use pixel_format::*;
use lcms2::*;
use rgb::*;
use image::*;
use convert::*;


pub fn load_image<P: AsRef<Path>>(path: P, opaque: bool) -> Result<Image, lodepng::Error> {
    let path = path.as_ref();
    let data = if path.as_os_str() == "-" {
        let mut data = Vec::new();
        std::io::stdin().read_to_end(&mut data)?;
        data
    } else {
        file::get(path)?
    };
    load_image_data(&data, opaque)
}

pub fn load_image_data(data: &[u8], opaque: bool) -> Result<Image, lodepng::Error> {

    if data.starts_with(b"\x89PNG") {
        png::load_png(data, opaque)
    }  else {
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
                Ok(rgb.to_image(profile, width, height, opaque))
            },
            mozjpeg::ColorSpace::JCS_CMYK => {
                let mut rgb: Vec<CMYK> = dinfo.read_scanlines().unwrap();
                Ok(rgb.to_image(profile, width, height, opaque))
            },
            mozjpeg::ColorSpace::JCS_GRAYSCALE => {
                let mut g: Vec<GRAY8> = dinfo.read_scanlines().unwrap();
                Ok(g.to_image(profile, width, height, opaque))
            },
            _ => Err(lodepng::Error(59)),
        }
    }
}

#[cfg(test)]
mod test_linear;
#[cfg(test)]
use bitmap::*;

#[cfg(test)]
fn convert(left: &Image) -> Bitmap<test_linear::RGBAPLU> {
    use test_linear::ToRGBAPLU;

    match left {
        &Image::RGB8(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
        &Image::RGBA8(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
        &Image::RGB16(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
        &Image::RGBA16(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
        &Image::GRAY8(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
        &Image::GRAY16(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
        &Image::GRAYA8(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
        &Image::GRAYA16(ref img) => Bitmap::new(img.bitmap.to_rgbaplu(), img.width, img.height),
    }
}

#[cfg(test)]
fn compare(left: &Image, right: &Image) -> f64 {
    let left = convert(left);
    let right = convert(right);
    left.bitmap
        .iter()
        .zip(right.bitmap.iter())
        .map(|(&a, &b)| {
            let d = a - b;
            (d.r*d.r + d.g*d.g + d.b*d.b + d.a*d.a) as f64
        })
        .sum::<f64>() / (4 * left.width * left.height) as f64
}

#[test]
fn image_gray() {
    let g0 = load_image("tests/gray1-rgba16.png", false).unwrap();
    let g1 = load_image("tests/gray1-rgba.png", false).unwrap();
    let g2 = load_image("tests/gray1-pal.png", false).unwrap();
    let g3 = load_image("tests/gray1-gray.png", false).unwrap();
    let g4 = load_image("tests/gray1.jpg", false).unwrap();
    let g5 = load_image("tests/gray1-rgba.png", true).unwrap();

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

    match (g1, g5) {
        (Image::RGBA16(_), Image::RGB16(_)) => {},
        _ => panic!("opaque flag is supposed to return non-alpha type"),
    }
}

#[test]
fn image_gray_profile() {

    let gp1 = load_image("tests/gray-profile.png", false).unwrap();
    let gp2 = load_image("tests/gray-profile2.png", false).unwrap();
    let gp3 = load_image("tests/gray-profile.jpg", false).unwrap();

    let diff = compare(&gp1, &gp2);
    assert!(diff < 0.0003, "{}", diff);

    let diff = compare(&gp1, &gp3);
    assert!(diff < 0.0003, "{}", diff);
}

#[test]
fn image_load1() {

    let prof_jpg = load_image("tests/profile.jpg", false).unwrap();
    let prof_png = load_image("tests/profile.png", false).unwrap();
    let diff = compare(&prof_jpg, &prof_png);
    assert!(diff <= 0.002);

    let strip_jpg = load_image("tests/profile-stripped.jpg", false).unwrap();
    let diff = compare(&strip_jpg, &prof_jpg);
    assert!(diff > 0.002, "{}", diff);

    let strip_png = load_image("tests/profile-stripped.png", false).unwrap();
    let diff = compare(&strip_jpg, &strip_png);
    assert!(diff > 0.002, "{}", diff);
}

#[test]
fn image_4bit() {

    let im1 = load_image("tests/tile1.png", false).unwrap();
    let im2 = load_image("tests/tile2.png", false).unwrap();
    let diff = compare(&im1, &im2);
    assert!(diff <= 0.00002);
}

#[test]
fn image_cmyk() {

    let im1 = load_image("tests/cmyk.png", true).unwrap();
    let im2 = load_image("tests/cmyk.jpg", true).unwrap();
    let diff = compare(&im1, &im2);
    assert!(diff <= 0.002);
}

#[test]
fn image_bw() {
    load_image("tests/1bit.png", false).unwrap();
}

#[test]
fn pngtestsuite() {
    for entry in std::fs::read_dir("tests/pngtestsuite").unwrap().filter_map(|p|p.ok()) {
        let path = entry.path();
        let filenamestr = path.file_name().unwrap().to_string_lossy();
        // ignore signature test, since fallback for jpeg panics
        // ignore checksum check, since lodepng doesn't do it
        if !filenamestr.ends_with(".png") || filenamestr.starts_with("xs") || filenamestr.starts_with("xcs") {
            continue;
        }
        let res = load_image(&path, false);
        if filenamestr.starts_with("x") {
            assert!(res.is_err());
        } else {
            res.unwrap();
        }
    }
}
