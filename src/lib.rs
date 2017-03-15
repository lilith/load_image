#![allow(unknown_lints)]

extern crate lodepng;
extern crate lcms2;
extern crate mozjpeg;
extern crate file;
extern crate rgb;

mod pixel_format;
mod bitmap;
mod endian;

use std::io::Read;
use std::path::Path;
use pixel_format::*;
use lcms2::*;
use rgb::*;
use bitmap::*;
use endian::*;


trait CopyAlpha<Converted: Copy> where Self: Copy {
    fn copy_alpha(src: &[Self], dst: &mut [Converted]);
}

macro_rules! copy_alpha_impl {
    ($in_type:ty => $out_type:ty, $fix:expr) => {
        impl CopyAlpha<$out_type> for $in_type {
            #[allow(redundant_closure_call)]
            fn copy_alpha(src: &[Self], dst: &mut [$out_type]) {
                for (s,d) in src.iter().zip(dst.iter_mut()) {
                    ($fix)(s,d);
                }
            }
        }
    }
}

macro_rules! copy_alpha_nop {
    ($in_type:ty => $out_type:ty) => {
        impl CopyAlpha<$out_type> for $in_type {
            fn copy_alpha(_: &[Self], _: &mut [$out_type]) {}
        }
    }
}

copy_alpha_nop!{ RGB8 => RGB16 }
copy_alpha_nop!{ RGB16 => RGB16 }
copy_alpha_impl!{ RGBA8 => RGBA16, |s:&RGBA8,d:&mut RGBA16|{d.a = s.a as u16 * 257} }
copy_alpha_impl!{ RGBA16 => RGBA16, |s:&RGBA16,d:&mut RGBA16|{d.a = s.a} }
copy_alpha_nop!{ lodepng::Grey<u8> => lodepng::Grey<u16> }
copy_alpha_nop!{ lodepng::Grey<u16> => lodepng::Grey<u16> }
copy_alpha_impl!{ lodepng::GreyAlpha<u8> => lodepng::GreyAlpha<u16>, |s:&lodepng::GreyAlpha<u8>,d:&mut lodepng::GreyAlpha<u16>|{d.1 = s.1 as u16 * 257} }
copy_alpha_impl!{ lodepng::GreyAlpha<u16> => lodepng::GreyAlpha<u16>, |s:&lodepng::GreyAlpha<u16>,d:&mut lodepng::GreyAlpha<u16>|{d.1 = s.1} }

trait ToSRGBImage {
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize, opaque: bool) -> Image;
}

trait Convertible<Converted: Copy> {
    fn apply_profile(&self, profile: Profile) -> Option<Vec<Converted>>;
}

impl ToSRGBImage for Vec<CMYK> {
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize, _opaque: bool) -> Image {
        let converted: Option<Vec<<pixel_format::CMYK as LcmsPixelConversion>::Converted>>;
        // The image may be CMYK, but lack any profile
        // The image may be CMYK, but with an RGB profile
        // The image may be CMYK with CMYK profile, but the profile may not work with LCMS
        // So in all cases fall back to a known good profile, since profile-less CMYK is bogus.
        converted = profile.and_then(|profile| self.apply_profile(profile)).or_else(||{
            self.apply_profile(Profile::new_icc(include_bytes!("cmyk.icc")).unwrap())
        });
        return (converted.unwrap(), width, height).into();
    }
}

impl<T> ToSRGBImage for [T]
    where T: LcmsPixelFormat + LcmsPixelConversion,
        T::Converted: LcmsPixelFormat + Default,
        T::ConvertedOpaque: LcmsPixelFormat + Default,
        Image: From<SizedVec<T>>,
        Image: From<SizedVec<T::Converted>>,
        Image: From<SizedVec<T::ConvertedOpaque>>,
        T: CopyAlpha<<T as LcmsPixelConversion>::Converted>
{
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize, opaque: bool) -> Image {
        if let Some(profile) = profile {
            if opaque {
                let converted: Option<Vec<T::ConvertedOpaque>> = self.apply_profile(profile);
                if let Some(pixels) = converted {
                    return (pixels, width, height).into();
                }
            } else {
                let converted: Option<Vec<T::Converted>> = self.apply_profile(profile);
                if let Some(mut pixels) = converted {
                    T::copy_alpha(self, &mut pixels);
                    return (pixels, width, height).into();
                }
            }
        }
        (self.to_owned(), width, height).into()
    }
}

impl<T, Converted> Convertible<Converted> for [T]
    where T: Copy + LcmsPixelFormat,
    Converted: Copy + LcmsPixelFormat + Default,
    Image: From<SizedVec<Converted>>,
{
    fn apply_profile(&self, profile: Profile) -> Option<Vec<Converted>> {
        let (format, color_space) = T::pixel_format();
        let (dest_format, _) = Converted::pixel_format();
        if profile.color_space() != color_space {
            return None;
        }
        let dest_profile = if color_space == ColorSpaceSignature::SigGrayData {
            Profile::new_icc(include_bytes!("gray.icc")).unwrap()
        } else {
            Profile::new_srgb()
        };

        let t = Transform::new(&profile, format, &dest_profile, dest_format, Intent::RelativeColorimetric);
        let mut dest:Vec<Converted> = vec![Default::default(); self.len()];

        t.transform_pixels(self, &mut dest);
        Some(dest)
    }
}

#[derive(Debug)]
pub enum Image {
    RGB8(Bitmap<RGB8>),
    RGBA8(Bitmap<RGBA8>),
    RGB16(Bitmap<RGB16>),
    RGBA16(Bitmap<RGBA16>),
    GRAY8(Bitmap<lodepng::Grey<u8>>),
    GRAY16(Bitmap<lodepng::Grey<u16>>),
    GRAYA8(Bitmap<lodepng::GreyAlpha<u8>>),
    GRAYA16(Bitmap<lodepng::GreyAlpha<u16>>),
}

macro_rules! image_from_vec {
    ($in_type:ty => $out_enum:path) => {
        impl From<SizedVec<$in_type>> for Image {
            fn from(f: SizedVec<$in_type>) -> Image {
                $out_enum(Bitmap::new(f.0, f.1, f.2))
            }
        }
    }
}

image_from_vec!{ RGB8 => Image::RGB8 }
image_from_vec!{ RGBA8 => Image::RGBA8 }
image_from_vec!{ RGB16 => Image::RGB16 }
image_from_vec!{ RGBA16 => Image::RGBA16 }
image_from_vec!{ lodepng::Grey<u8> => Image::GRAY8 }
image_from_vec!{ lodepng::Grey<u16> => Image::GRAY16 }
image_from_vec!{ lodepng::GreyAlpha<u8> => Image::GRAYA8 }
image_from_vec!{ lodepng::GreyAlpha<u16> => Image::GRAYA16 }

fn from_palette<T: Copy>(buf: &[u8], pal: &[T], bitdepth: u8, width: usize, height: usize) -> Option<Bitmap<T>> {
    match bitdepth {
        8 => Some(Bitmap::new(buf.iter().map(|&c| pal[c as usize]).collect(), width, height)),
        depth @ 1 | depth @ 2 | depth @ 4 => {
            let px_per_byte = 8 / depth;
            let mask = (1<<depth) - 1;
            Some(Bitmap::new(buf.iter()
                                 .flat_map(|c| (0..px_per_byte).rev().map(move |n| pal[(c >> (n * depth) & mask) as usize]))
                                 .take(width * height)
                                 .collect(),
                             width,
                             height))
        },
        _ => None,
    }
}

fn load_png(mut state: lodepng::State, res: lodepng::Image, opaque: bool) -> Result<Image, lodepng::Error> {

    let profile = if state.info_png().get("sRGB").is_some() {
        None
    } else if let Ok(iccp) = state.get_icc() {
        Profile::new_icc(iccp.as_ref())
    } else {
        None
    };

    match res {
        lodepng::Image::RGBA(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RGB(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RGB16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RGBA16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::Grey(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::Grey16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::GreyAlpha(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::GreyAlpha16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height, opaque)),
        lodepng::Image::RawData(rawdata) => {
            let mut png = state.info_raw_mut();
            let depth = png.bitdepth as u8;
            if png.colortype() == lodepng::LCT_PALETTE {
                let pal = png.palette_mut();
                let ncolors = pal.len();
                return match pal.to_image(profile, 1, ncolors, opaque) {
                    Image::RGBA8(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGBA8).ok_or(lodepng::Error(59)),
                    Image::RGBA16(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGBA16).ok_or(lodepng::Error(59)),
                    _ => Err(lodepng::Error(59)),
                };
            }
            Err(lodepng::Error(59))
        },
    }
}

pub fn load_image<P: AsRef<Path>>(path: P, opaque: bool) -> Result<Image, lodepng::Error> {
    let path = path.as_ref();
    let data = if path.as_os_str() == "-" {
        let mut data = Vec::new();
        std::io::stdin().read_to_end(&mut data)?;
        data
    } else {
        file::get(path)?
    };

    let mut state = lodepng::State::new();
    state.color_convert(false);
    state.remember_unknown_chunks(true);

    if data.starts_with(b"\x89PNG") {
        let img = state.decode(&data)?;
        load_png(state, img, opaque)
    }  else {
        let mut dinfo = mozjpeg::Decompress::new();
        dinfo.set_mem_src(&data[..]);
        dinfo.save_marker(mozjpeg::Marker::APP(2));
        assert!(dinfo.read_header(true));
        assert!(dinfo.start_decompress());
        let width = dinfo.output_width();
        let height = dinfo.output_height();

        let profile = if let Some(marker) = dinfo.markers().next() {
            let data = marker.data;
            if b"ICC_PROFILE\0" == &data[0..12] {
                let icc = &data[14..];
                Profile::new_icc(icc)
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
                let mut g: Vec<lodepng::Grey<u8>> = dinfo.read_scanlines().unwrap();
                Ok(g.to_image(profile, width, height, opaque))
            },
            _ => Err(lodepng::Error(59)),
        }
    }
}

#[cfg(test)]
mod test_linear;

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
