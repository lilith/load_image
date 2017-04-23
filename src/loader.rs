
use std::io;
use std::io::Read;
use std::path::Path;
use image::*;
use lodepng;
use png;
use jpeg;
use file;

pub struct Loader {
    opaque: bool,
}

impl Loader {
    #[inline]
    pub fn new() -> Self {
        Loader {
            opaque: false,
        }
    }

    #[inline]
    pub fn opaque(&mut self, v: bool) -> &mut Self {
        self.opaque = v;
        self
    }

    pub fn load_path<P: AsRef<Path>>(&self, path: P) -> Result<Image, lodepng::Error> {
        let path = path.as_ref();
        let data = if path.as_os_str() == "-" {
            let mut data = Vec::new();
            io::stdin().read_to_end(&mut data)?;
            data
        } else {
            file::get(path)?
        };
        self.load_data(&data)
    }

    pub fn load_data(&self, data: &[u8]) -> Result<Image, lodepng::Error> {
        if data.starts_with(b"\x89PNG") {
            png::load_png(data, self.opaque)
        } else if data[0] == 0xFF {
            jpeg::load_jpeg(data)
        } else {
            Err(lodepng::Error(28))
        }
    }
}
