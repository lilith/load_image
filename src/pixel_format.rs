use lcms2::*;
use rgb::alt::*;
use rgb::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CMYK {
    pub c: u8,
    pub m: u8,
    pub y: u8,
    pub k: u8,
}

unsafe impl rgb::Pod for CMYK {}
unsafe impl rgb::Zeroable for CMYK {}

pub trait LcmsPixelFormat where Self: Copy {
    fn pixel_format() -> (PixelFormat, ColorSpaceSignature);
}

pub trait LcmsPixelConversion where Self: Copy {
    type Converted: Copy;
    type ConvertedOpaque: Copy;
}

macro_rules! pixel_conversion {
    ( $in_type:ty => $out_type:ty, $out_type_opaque:ty ) => {
        impl LcmsPixelConversion for $in_type {
            type Converted = $out_type;
            type ConvertedOpaque = $out_type_opaque;
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

pixel_format!{CMYK, PixelFormat::CMYK_8_REV, ColorSpaceSignature::CmykData }
pixel_conversion!{CMYK => RGB16, RGB16}

pixel_format!{RGB8, PixelFormat::RGB_8, ColorSpaceSignature::RgbData }
pixel_format!{RGBA8, PixelFormat::RGBA_8, ColorSpaceSignature::RgbData }
pixel_format!{Gray<u8>, PixelFormat::GRAY_8, ColorSpaceSignature::GrayData }
pixel_format!{GrayAlpha<u8>, PixelFormat::GRAYA_8, ColorSpaceSignature::GrayData }
pixel_format!{RGB16, PixelFormat::RGB_16, ColorSpaceSignature::RgbData }
pixel_format!{RGBA16, PixelFormat::RGBA_16, ColorSpaceSignature::RgbData }
pixel_format!{Gray<u16>, PixelFormat::GRAY_16, ColorSpaceSignature::GrayData }
pixel_format!{GrayAlpha<u16>, PixelFormat::GRAYA_16, ColorSpaceSignature::GrayData }

pixel_conversion!{RGB8 => RGB16, RGB16}
pixel_conversion!{RGBA8 => RGBA16, RGB16}
pixel_conversion!{Gray<u8> => Gray<u16>, Gray<u16>}
pixel_conversion!{GrayAlpha<u8> => GrayAlpha<u16>, Gray<u16>}
pixel_conversion!{RGB16 => RGB16, RGB16}
pixel_conversion!{RGBA16 => RGBA16, RGB16}
pixel_conversion!{Gray<u16> => Gray<u16>, Gray<u16>}
pixel_conversion!{GrayAlpha<u16> => GrayAlpha<u16>, Gray<u16>}
