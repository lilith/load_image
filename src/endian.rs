use rgb::alt::*;
use rgb::*;

pub trait NativeEndian {
    fn to_native(&mut self) -> &mut Self;
}

impl NativeEndian for [RGB16] {
    #[inline]
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
    #[inline]
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

impl NativeEndian for [GrayAlpha<u16>] {
    fn to_native(&mut self) -> &mut Self {
        for n in self.iter_mut() {
            *n = GrayAlpha(u16::from_be(n.0), u16::from_be(n.1));
        }
        self
    }
}

impl NativeEndian for [Gray<u16>] {
    fn to_native(&mut self) -> &mut Self {
        for n in self.iter_mut() {
            *n = Gray(u16::from_be(n.0));
        }
        self
    }
}
