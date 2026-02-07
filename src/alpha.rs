// Helper functions for checking alpha transparency
// Works with both rgb 0.8.52 and 0.8.91+ by using type aliases

#[inline]
pub fn is_opaque<T>(bitmap: &[T]) -> bool 
where
    T: HasAlphaChannel,
{
    T::is_opaque_slice(bitmap)
}

// Private trait for internal dispatch
trait HasAlphaChannel: Sized {
    fn is_opaque_slice(bitmap: &[Self]) -> bool;
}

impl HasAlphaChannel for rgb::RGBA8 {
    #[inline]
    fn is_opaque_slice(bitmap: &[Self]) -> bool {
        !bitmap.iter().any(|px| px.a != 255)
    }
}

impl HasAlphaChannel for rgb::RGBA16 {
    #[inline]
    fn is_opaque_slice(bitmap: &[Self]) -> bool {
        !bitmap.iter().any(|px| px.a != 65535)
    }
}

impl HasAlphaChannel for rgb::GrayAlpha<u8> {
    #[inline]
    fn is_opaque_slice(bitmap: &[Self]) -> bool {
        !bitmap.iter().any(|px| px.a != 255)
    }
}

impl HasAlphaChannel for rgb::GrayAlpha<u16> {
    #[inline]
    fn is_opaque_slice(bitmap: &[Self]) -> bool {
        !bitmap.iter().any(|px| px.a != 65535)
    }
}

#[test]
fn alphapx() {
    use rgb::*;
    
    let a = vec![RGBA8::new(0, 0, 0, 255)];
    assert!(is_opaque(&a));

    let a = vec![GrayAlpha(0u8, 255)];
    assert!(is_opaque(&a));
    let a = vec![GrayAlpha(0u8, 254)];
    assert!(!is_opaque(&a));

    let a = vec![RGBA16::new(0, 0, 0, 255)];
    assert!(!is_opaque(&a));
    let a = vec![RGBA16::new(0, 0, 0, 65535)];
    assert!(is_opaque(&a));
}
