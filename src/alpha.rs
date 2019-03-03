use rgb::alt::*;
use rgb::*;

pub trait IsTransparentPixel {
    fn is_transparent(&self) -> bool;
}

impl IsTransparentPixel for RGBA8 {
    fn is_transparent(&self) -> bool {
        self.a != 255
    }
}

impl IsTransparentPixel for RGBA16 {
    fn is_transparent(&self) -> bool {
        self.a != 65535
    }
}

impl IsTransparentPixel for GrayAlpha<u8> {
    fn is_transparent(&self) -> bool {
        self.1 != 255
    }
}

impl IsTransparentPixel for GrayAlpha<u16> {
    fn is_transparent(&self) -> bool {
        self.1 != 65535
    }
}

pub fn is_opaque<T>(bitmap: &[T]) -> bool where T: IsTransparentPixel {
    !bitmap.iter().any(IsTransparentPixel::is_transparent)
}

#[test]
fn alphapx() {
    let a = vec![RGBA8::new(0,0,0,255)];
    assert!(is_opaque(&a));

    let a = vec![GrayAlpha(0u8,255)];
    assert!(is_opaque(&a));
    let a = vec![GrayAlpha(0u8,254)];
    assert!(!is_opaque(&a));

    let a = vec![RGBA16::new(0,0,0,255)];
    assert!(!is_opaque(&a));
    let a = vec![RGBA16::new(0,0,0,65535)];
    assert!(is_opaque(&a));
}
