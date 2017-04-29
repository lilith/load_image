
use std::io;
use std::io::Read;
use std::path::Path;
use image::*;
use lcms2::*;
use lodepng;
use file;

#[derive(Eq, PartialEq)]
pub enum Profiles {
    All,
    None,
    NonsRGB,
}

pub struct Loader {
    pub opaque: bool, // FIXME: pub(crate)
    pub profiles: Profiles, // FIXME: pub(crate)
}

impl Loader {
    #[inline]
    pub fn new() -> Self {
        Loader {
            opaque: false,
            profiles: Profiles::NonsRGB,
        }
    }

    #[inline]
    pub fn opaque(&mut self, v: bool) -> &mut Self {
        self.opaque = v;
        self
    }

    #[inline]
    pub fn profiles(&mut self, v: Profiles) -> &mut Self {
        self.profiles = v;
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
            self.load_png(data)
        } else if data[0] == 0xFF {
            self.load_jpeg(data)
        } else {
            Err(lodepng::Error(28))
        }
    }

    pub fn process_profile(&self, profile: Result<Profile, ()>) -> Option<Profile> {
        match profile {
            Err(_) => None,
            Ok(profile) => {
                if self.profiles == Profiles::NonsRGB {
                    if let Some(desc) = profile.info(InfoType::Description, "en", "US") {
                        if desc.starts_with("sRGB ") {
                            return None;
                        }
                    }
                }
                if self.profiles == Profiles::None {
                    return None;
                }
                Some(profile)
            }
        }
    }
}
