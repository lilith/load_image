
use std::io;
use std::io::Read;
use std::fs;
use std::path::Path;
use image::*;
use lcms2::*;
use lodepng;

#[derive(Eq, PartialEq)]
pub enum Profiles {
    All,
    None,
    NonsRGB,
}

pub struct Loader {
    pub(crate) opaque: bool,
    pub(crate) profiles: Profiles,
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
    pub fn profiles(&mut self, convert_profiles: Profiles) -> &mut Self {
        self.profiles = convert_profiles;
        self
    }

    pub fn load_path<P: AsRef<Path>>(&self, path: P) -> Result<Image, lodepng::Error> {
        let path = path.as_ref();
        let mut data = Vec::new();
        let (data, stat) = if path.as_os_str() == "-" {
            io::stdin().read_to_end(&mut data)?;
            (data, None)
        } else {
            let mut file = fs::File::open(path)?;
            let stat = file.metadata()?;
            file.read_to_end(&mut data)?;
            (data, Some(stat))
        };
        self.load_data_with_stat(&data, stat)
    }

    pub fn load_data(&self, data: &[u8]) -> Result<Image, lodepng::Error> {
        self.load_data_with_stat(data, None)
    }

    pub fn load_data_with_stat(&self, data: &[u8], meta: Option<fs::Metadata>) -> Result<Image, lodepng::Error> {
        if data.starts_with(b"\x89PNG") {
            self.load_png(data, meta)
        } else if data[0] == 0xFF {
            self.load_jpeg(data, meta)
        } else {
            Err(lodepng::Error(28))
        }
    }

    pub fn process_profile(&self, profile: LCMSResult<Profile>) -> Option<Profile> {
        match profile {
            Err(_) => None,
            Ok(profile) => {
                if self.profiles == Profiles::NonsRGB {
                    if let Some(desc) = profile.info(InfoType::Description, Locale::new("en_US")) {
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
