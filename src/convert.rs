use rgb::*;
use image::*;
use imgref::*;
use lcms2::*;
use pixel_format::*;

pub trait CopyAlpha<Converted: Copy> where Self: Copy {
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
copy_alpha_nop!{ GRAY8 => GRAY16 }
copy_alpha_nop!{ GRAY16 => GRAY16 }
copy_alpha_impl!{ GRAYA8 => GRAYA16, |s:&GRAYA8,d:&mut GRAYA16|{d.1 = s.1 as u16 * 257} }
copy_alpha_impl!{ GRAYA16 => GRAYA16, |s:&GRAYA16,d:&mut GRAYA16|{d.1 = s.1} }

pub trait ToSRGBImage {
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize, opaque: bool) -> Image;
}

pub trait Convertible<Converted: Copy> {
    fn apply_profile(&self, profile: Profile) -> Option<Vec<Converted>>;
}

impl ToSRGBImage for Vec<CMYK> {
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize, _opaque: bool) -> Image {
        let converted: Option<Vec<<CMYK as LcmsPixelConversion>::Converted>>;
        // The image may be CMYK, but lack any profile
        // The image may be CMYK, but with an RGB profile
        // The image may be CMYK with CMYK profile, but the profile may not work with LCMS
        // So in all cases fall back to a known good profile, since profile-less CMYK is bogus.
        converted = profile.and_then(|profile| self.apply_profile(profile)).or_else(||{
            self.apply_profile(Profile::new_icc(include_bytes!("cmyk.icc")).unwrap())
        });
        ImgVec::new(converted.unwrap(), width, height).into()
    }
}

impl<T> ToSRGBImage for [T]
    where T: LcmsPixelFormat + LcmsPixelConversion,
        T::Converted: LcmsPixelFormat + Default,
        T::ConvertedOpaque: LcmsPixelFormat + Default,
        Image: From<ImgVec<T>>,
        Image: From<ImgVec<T::Converted>>,
        Image: From<ImgVec<T::ConvertedOpaque>>,
        T: CopyAlpha<<T as LcmsPixelConversion>::Converted>
{
    fn to_image(&mut self, profile: Option<Profile>, width: usize, height: usize, opaque: bool) -> Image {
        if let Some(profile) = profile {
            if opaque {
                let converted: Option<Vec<T::ConvertedOpaque>> = self.apply_profile(profile);
                if let Some(pixels) = converted {
                    return ImgVec::new(pixels, width, height).into();
                }
            } else {
                let converted: Option<Vec<T::Converted>> = self.apply_profile(profile);
                if let Some(mut pixels) = converted {
                    T::copy_alpha(self, &mut pixels);
                    return ImgVec::new(pixels, width, height).into();
                }
            }
        }
        ImgVec::new(self.to_owned(), width, height).into()
    }
}

impl<T, Converted> Convertible<Converted> for [T]
    where T: Copy + LcmsPixelFormat,
    Converted: Copy + LcmsPixelFormat + Default,
    Image: From<ImgVec<Converted>>,
{
    fn apply_profile(&self, profile: Profile) -> Option<Vec<Converted>> {
        let (format, color_space) = T::pixel_format();
        let (dest_format, _) = Converted::pixel_format();
        if profile.color_space() != color_space {
            return None;
        }
        if color_space == ColorSpaceSignature::SigRgbData {
            if let Some(desc) = profile.info(InfoType::Description, "en", "US") {
                if desc.starts_with("sRGB ") {
                    return None;
                }
            }
        }
        let dest_profile = if color_space == ColorSpaceSignature::SigGrayData {
            Profile::new_icc(include_bytes!("gray.icc")).unwrap()
        } else {
            Profile::new_srgb()
        };

        match Transform::new(&profile, format, &dest_profile, dest_format, Intent::RelativeColorimetric) {
            Ok(t) => {
                let mut dest:Vec<Converted> = vec![Default::default(); self.len()];

                t.transform_pixels(self, &mut dest);
                Some(dest)
            }
            _ => None
        }
    }
}

impl From<Image> for Img<ImageData> {
    fn from(img: Image) -> Self {
        Img::new(img.bitmap, img.width, img.height)
    }
}

macro_rules! impl_img {
    ($px:ident) => {
        impl From<ImgVec<$px>> for Image {
            fn from(bitmap: ImgVec<$px>) -> Image {
                Image {
                    width: bitmap.width(),
                    height: bitmap.height(),
                    bitmap: ImageData::$px(bitmap.buf),
                }
            }
        }
    }
}

impl_img!(RGB8);
impl_img!(RGBA8);
impl_img!(RGB16);
impl_img!(RGBA16);
impl_img!(GRAY8);
impl_img!(GRAY16);
impl_img!(GRAYA8);
impl_img!(GRAYA16);
