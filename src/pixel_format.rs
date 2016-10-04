use lcms2::*;
use rgb::*;
extern crate lodepng;

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
pixel_conversion!{RGB8 => RGB16, RGB16}
pixel_conversion!{RGB16 => RGB16, RGB16}
pixel_conversion!{RGBA8 => RGBA16, RGB16}
pixel_conversion!{RGBA16 => RGBA16, RGB16}
pixel_conversion!{lodepng::Grey<u8> => lodepng::Grey<u16>, lodepng::Grey<u16>}
pixel_conversion!{lodepng::Grey<u16> => lodepng::Grey<u16>, lodepng::Grey<u16>}
pixel_conversion!{lodepng::GreyAlpha<u8> => lodepng::GreyAlpha<u16>, lodepng::Grey<u16>}
pixel_conversion!{lodepng::GreyAlpha<u16> => lodepng::GreyAlpha<u16>, lodepng::Grey<u16>}
