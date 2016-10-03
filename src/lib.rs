
extern crate lodepng;
extern crate lcms2;
extern crate mozjpeg;
extern crate file;
extern crate rgb;


use std::io::Read;
use lcms2::*;
use rgb::*;


#[derive(Debug)]
pub struct Bitmap<T> {
    pub bitmap: Vec<T>,
    pub width: usize,
    pub height: usize,
}


#[derive(Debug, Copy, Clone)]
pub struct BitmapRef<'a, T: 'a> {
    pub bitmap: &'a [T],
    pub width: usize,
    pub height: usize,
}

impl<'a, T> BitmapRef<'a, T> {
    pub fn new(bitmap: &'a [T], width: usize, height: usize) -> BitmapRef<'a, T> {
        BitmapRef {
            bitmap: bitmap,
            width: width,
            height: height,
        }
    }
}

impl<T> Bitmap<T> {
    pub fn new(bitmap: Vec<T>, width: usize, height: usize) -> Bitmap<T> {
        Bitmap {
            bitmap: bitmap,
            width: width,
            height: height,
        }
    }

    pub fn new_ref(&self) -> BitmapRef<T> {
        BitmapRef::new(self.bitmap.as_ref(), self.width, self.height)
    }
}

trait LcmsPixelFormat where Self: Copy {
    fn pixel_format() -> (PixelFormat, ColorSpaceSignature);
}

trait LcmsPixelConversion where Self: Copy {
    type Converted: Copy;
}

trait CopyAlpha<Converted: Copy> where Self: Copy {
    fn copy_alpha(src: &[Self], dst: &mut [Converted]);
}

macro_rules! copy_alpha_impl {
    ($in_type:ty => $out_type:ty, $fix:expr) => {
        impl CopyAlpha<$out_type> for $in_type {
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

macro_rules! pixel_conversion {
    ( $in_type:ty => $out_type:ty ) => {
        impl LcmsPixelConversion for $in_type {
            type Converted = $out_type;
        }
    };
}

macro_rules! pixel_format {
    ( $in_type:ty, $format:expr, $colorspace:expr ) => {
        impl LcmsPixelFormat for $in_type {
            fn pixel_format() -> (PixelFormat, ColorSpaceSignature) {
                ($format, $colorspace)
            }
        }
    };
}

// assumes LE CPU :(
pixel_format!{RGB8, PixelFormat::RGB_8, ColorSpaceSignature::SigRgbData }
pixel_format!{RGB16, PixelFormat::RGB_16, ColorSpaceSignature::SigRgbData }
pixel_format!{RGBA8, PixelFormat::RGBA_8, ColorSpaceSignature::SigRgbData }
pixel_format!{RGBA16, PixelFormat::RGBA_16, ColorSpaceSignature::SigRgbData }
pixel_format!{lodepng::Grey<u8>, PixelFormat::GRAY_8, ColorSpaceSignature::SigGrayData }
pixel_format!{lodepng::Grey<u16>, PixelFormat::GRAY_16, ColorSpaceSignature::SigGrayData }
pixel_format!{lodepng::GreyAlpha<u8>, PixelFormat::GRAYA_8, ColorSpaceSignature::SigGrayData }
pixel_format!{lodepng::GreyAlpha<u16>, PixelFormat::GRAYA_16, ColorSpaceSignature::SigGrayData }

// assumes LE CPU :(
pixel_conversion!{RGB8 => RGB16}
pixel_conversion!{RGB16 => RGB16}
pixel_conversion!{RGBA8 => RGBA16}
pixel_conversion!{RGBA16 => RGBA16}
pixel_conversion!{lodepng::Grey<u8> => lodepng::Grey<u16>}
pixel_conversion!{lodepng::Grey<u16> => lodepng::Grey<u16>}
pixel_conversion!{lodepng::GreyAlpha<u8> => lodepng::GreyAlpha<u16>}
pixel_conversion!{lodepng::GreyAlpha<u16> => lodepng::GreyAlpha<u16>}

trait ToSRGBImage {
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize) -> Image;
}

impl<T> ToSRGBImage for [T]
    where T: Copy + LcmsPixelFormat + LcmsPixelConversion, T: std::fmt::Debug,
        T::Converted: Copy + LcmsPixelFormat + std::fmt::Debug,
        Image: From<(Vec<T::Converted>, usize, usize)>,
        Image: From<(Vec<T>, usize, usize)>,
        T: CopyAlpha<<T as LcmsPixelConversion>::Converted>
{
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize) -> Image {
        if let Some(profile) = profile {
            let (format, _) = T::pixel_format();
            let (dest_format, color_space) = T::Converted::pixel_format();
            if profile.color_space() == color_space {
                let dest_profile = if color_space == ColorSpaceSignature::SigRgbData {
                    Profile::new_srgb()
                } else {
                    Profile::new_icc(include_bytes!("gray.icc")).unwrap()
                };
                let t = Transform::new(&profile, format, &dest_profile, dest_format, Intent::RelativeColorimetric);
                let mut dest:Vec<T::Converted> = vec![unsafe { std::mem::zeroed() }; self.len()];

                t.transform_pixels(self, &mut dest);
                T::copy_alpha(self, &mut dest);
                return (dest, width, height).into();
            }
        }
        (self.to_owned(), width, height).into()
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
        impl From<(Vec<$in_type>, usize, usize)> for Image {
            fn from(f: (Vec<$in_type>, usize, usize)) -> Image {
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
            let pixels = 8 / depth;
            let mask = depth - 1;
            Some(Bitmap::new(buf.iter()
                                 .flat_map(|c| (0..pixels).rev().map(move |n| pal[(c >> (n * depth) & mask) as usize]))
                                 .take(width * height)
                                 .collect(),
                             width,
                             height))
        },
        _ => None,
    }
}

trait NativeEndian {
    fn to_native(&mut self) -> &mut Self;
}

impl NativeEndian for [RGB16] {
    fn to_native(&mut self) -> &mut Self {
        for n in self.iter_mut() {
            *n = RGB {
                r: u16::from_be(n.r),
                g: u16::from_be(n.g),
                b: u16::from_be(n.b),
            };
        }
        self
    }
}

impl NativeEndian for [RGBA16] {
    fn to_native(&mut self) -> &mut Self {
        for n in self.iter_mut() {
            *n = RGBA {
                r: u16::from_be(n.r),
                g: u16::from_be(n.g),
                b: u16::from_be(n.b),
                a: u16::from_be(n.a),
            };
        }
        self
    }
}

impl NativeEndian for [lodepng::GreyAlpha<u16>] {
    fn to_native(&mut self) -> &mut Self {
        for n in self.iter_mut() {
            *n = lodepng::GreyAlpha(u16::from_be(n.0), u16::from_be(n.1));
        }
        self
    }
}

impl NativeEndian for [lodepng::Grey<u16>] {
    fn to_native(&mut self) -> &mut Self {
        for n in self.iter_mut() {
            *n = lodepng::Grey(u16::from_be(n.0));
        }
        self
    }
}

fn load_png(mut state: lodepng::State, res: lodepng::Image) -> Result<Image, lodepng::Error> {

    let profile = if state.info_png().get("sRGB").is_some() {
        None
    } else if let Ok(iccp) = state.get_icc() {
        Profile::new_icc(iccp.as_ref())
    } else {
        None
    };

    match res {
        lodepng::Image::RGBA(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height)),
        lodepng::Image::RGB(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height)),
        lodepng::Image::RGB16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height)),
        lodepng::Image::RGBA16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height)),
        lodepng::Image::Grey(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height)),
        lodepng::Image::Grey16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height)),
        lodepng::Image::GreyAlpha(mut image) => Ok(image.buffer.as_mut().to_image(profile, image.width, image.height)),
        lodepng::Image::GreyAlpha16(mut image) => Ok(image.buffer.as_mut().to_native().to_image(profile, image.width, image.height)),
        lodepng::Image::RawData(rawdata) => {
            let mut png = state.info_raw_mut();
            let depth = png.bitdepth as u8;
            if png.colortype() == lodepng::LCT_PALETTE {
                let pal = png.palette_mut();
                let ncolors = pal.len();
                return match pal.to_image(profile, 1, ncolors) {
                    Image::RGBA8(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGBA8).ok_or(lodepng::Error(59)),
                    Image::RGBA16(pal) => from_palette(rawdata.buffer.as_ref(), &pal.bitmap, depth, rawdata.width, rawdata.height).map(Image::RGBA16).ok_or(lodepng::Error(59)),
                    _ => Err(lodepng::Error(59)),
                };
            }
            Err(lodepng::Error(59))
        },
    }
}

pub fn load_image(path: &str) -> Result<Image, lodepng::Error> {
    let data = match path {
        "-" => {
            let mut data = Vec::new();
            try!(std::io::stdin().read_to_end(&mut data));
            data
        },
        path => try!(file::get(path)),
    };

    let mut state = lodepng::State::new();
    state.color_convert(false);
    state.remember_unknown_chunks(true);

    match state.decode(&data) {
        Ok(img) => load_png(state, img),
        _ => {
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
                    Ok(rgb.to_image(profile, width, height))
                },
                mozjpeg::ColorSpace::JCS_GRAYSCALE => {
                    let mut g: Vec<lodepng::Grey<u8>> = dinfo.read_scanlines().unwrap();
                    Ok(g.to_image(profile, width, height))
                },
                _ => Err(lodepng::Error(59)),
            }
        },
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
    let g0 = load_image("tests/gray1-rgba16.png").unwrap();
    let g1 = load_image("tests/gray1-rgba.png").unwrap();
    let g2 = load_image("tests/gray1-pal.png").unwrap();
    let g3 = load_image("tests/gray1-gray.png").unwrap();
    let g4 = load_image("tests/gray1.jpg").unwrap();

    let diff = compare(&g0, &g1);
    assert!(diff < 0.00001, "{}", diff);

    let diff = compare(&g1, &g2);
    assert!(diff < 0.00001, "{}", diff);

    let diff = compare(&g1, &g3);
    assert!(diff < 0.00001, "{}", diff);

    let diff = compare(&g1, &g4);
    assert!(diff < 0.00006, "{}", diff);
}

#[test]
fn image_gray_profile() {

    let gp1 = load_image("tests/gray-profile.png").unwrap();
    let gp2 = load_image("tests/gray-profile2.png").unwrap();
    let gp3 = load_image("tests/gray-profile.jpg").unwrap();

    let diff = compare(&gp1, &gp2);
    assert!(diff < 0.0003, "{}", diff);

    let diff = compare(&gp1, &gp3);
    assert!(diff < 0.0003, "{}", diff);
}

#[test]
fn image_load1() {

    let prof_jpg = load_image("tests/profile.jpg").unwrap();
    let prof_png = load_image("tests/profile.png").unwrap();
    let diff = compare(&prof_jpg, &prof_png);
    assert!(diff <= 0.002);

    let strip_jpg = load_image("tests/profile-stripped.jpg").unwrap();
    let diff = compare(&strip_jpg, &prof_jpg);
    assert!(diff > 0.002, "{}", diff);

    let strip_png = load_image("tests/profile-stripped.png").unwrap();
    let diff = compare(&strip_jpg, &strip_png);
    assert!(diff > 0.002, "{}", diff);
}
