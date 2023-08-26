use std::io;
use std::collections::TryReserveError;
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    #[non_exhaustive]
    pub enum Error {
        Png(err: lodepng::Error) {
            from()
            display("lodepng: {}", err)
            source(err)
        }
        #[cfg(all(feature = "jpeg", not(feature = "mozjpeg")))]
        Jpeg(err: jpeg_decoder::Error) {
            display("jpeg-decoder: {}", err)
            source(err)
        }
        Io(err: io::Error) {
            from()
            display("I/O: {}", err)
            source(err)
            from(_e: TryReserveError) -> (io::Error::new(io::ErrorKind::OutOfMemory, "OOM"))
        }
        #[cfg(feature = "webp")]
        WebP {
            display("WebP decoding failed")
        }
        ImageTooLarge {
            display("Image is larger than the allowed maximum dimension")
        }
        UnsupportedJpeg {
            display("Reading of JPEG header failed")
        }
        UnsupportedFileFormat {
            display("This file doesn't look like any of the supported image formats")
        }
    }
}

#[cfg(all(feature = "jpeg", not(feature = "mozjpeg")))]
impl From<jpeg_decoder::Error> for Error {
    #[cold]
    fn from(e: jpeg_decoder::Error) -> Self {
        Self::Jpeg(e)
    }
}
