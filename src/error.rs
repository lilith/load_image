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
        #[cfg(feature = "jpeg")]
        Jpeg(err: jpeg_decoder::Error) {
            from()
            display("jpeg-decoder: {}", err)
            source(err)
        }
        Io(err: io::Error) {
            from()
            display("I/O: {}", err)
            source(err)
            from(_e: TryReserveError) -> (io::Error::new(io::ErrorKind::OutOfMemory, "OOM"))
        }
        ImageTooLarge {
            display("Max is 10K pixels")
        }
        UnsupportedJpeg {
            display("Reading of JPEG header failed")
        }
        UnsupportedFileFormat {
            display("This file doesn't look like any of the supported image formats")
        }
    }
}
