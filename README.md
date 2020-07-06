# Load image as sRGB

Glue code for a few libraries that correctly loads a JPEG or PNG image into memory, taking into accout color profile metadata in PNG chunks, EXIF data and app markers. Converts CMYK to RGB if needed.

For Rust 1.43 or later.


```toml
[dependencies]
load_image = "2"
```

```rust
fn main() {
    let path = std::env::args().nth(1).expect("File name");
    // the flag is for removing alpha channel
    let img = load_image::load_image(path, false).unwrap();
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

The `load_image` function doesn't panic, but it uses unwinding internally. Error handling won't work in crates compiled with the `panic = "abort"` option.
