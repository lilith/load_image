# Load image as sRGB

Glue code for a few libraries that correctly loads a JPEG, PNG, or (optionally) AVIF image into memory, taking into accout color profile metadata in PNG chunks, EXIF data and app markers. Converts CMYK to RGB if needed.

```bash
cargo add load_image
```

```rust
fn main() {
    let path = std::env::args().nth(1).expect("File name");
    let img = load_image::load_image(path).unwrap();
}
```

The returned `Image` is:

```rust
struct Image {
    pub width: usize,
    pub height: usize,
    pub bitmap: enum ImageData {
        RGB8(Vec<RGB8>),
        RGBA8(Vec<RGBA8>),
        RGB16(Vec<RGB16>),
        RGBA16(Vec<RGBA16>),
        GRAY8(Vec<GRAY8>),
        GRAY16(Vec<GRAY16>),
        GRAYA8(Vec<GRAYA8>),
        GRAYA16(Vec<GRAYA16>),
    }
}
```

The bitmap is packed, so `x + y * width` gives the pixel at `x,y`.

The `load_image` function doesn't panic, but if you enable the [`mozjpeg` feature](https://lib.rs/crates/load_image/features), it will depend on unwinding internally, and won't be compatible with crates compiled with `panic = "abort"` option.
