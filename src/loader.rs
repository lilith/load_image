use crate::image::*;
use lcms2::*;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

#[derive(Eq, PartialEq)]
pub enum Profiles {
    /// Apply all profiles
    All,
    /// Do not support profiles (gives incorrectly-looking images, but doesn't change pixel values)
    None,
    /// Apply profiles only if they don't appear to be sRGB
    NonsRGB,
}

pub struct Loader {
    pub(crate) discard_alpha: bool,
    pub(crate) metadata: bool,
    pub(crate) profiles: Profiles,
}

impl Loader {
    #[inline(always)]
    pub fn new() -> Self {
        Loader {
            discard_alpha: false,
            metadata: false,
            profiles: Profiles::NonsRGB,
        }
    }

    /// If true, alpha channel will be discarded.
    /// Default is false, which supports transparency.
    #[inline(always)]
    pub fn opaque(&mut self, discard_alpha: bool) -> &mut Self {
        self.discard_alpha = discard_alpha;
        self
    }

    /// If true, will keep all image metadata.
    /// If false, it will apply color profiles, EXIF rotation, and discard everything else.
    #[inline(always)]
    pub fn metadata(&mut self, keep_metadata: bool) -> &mut Self {
        self.metadata = keep_metadata;
        self
    }

    /// Strategy for converting color profiles
    #[inline(always)]
    pub fn profiles(&mut self, convert_profiles: Profiles) -> &mut Self {
        self.profiles = convert_profiles;
        self
    }

    /// `-` is treated as stdin
    pub fn load_path<P: AsRef<Path>>(&self, path: P) -> Result<Image, crate::Error> {
        let path = path.as_ref();
        let mut data = Vec::new();
        let (data, stat) = if path.as_os_str() == "-" {
            fallible_collections::FallibleVec::try_reserve(&mut data, 1<<16)?; // arbitrary, better than 0
            io::stdin().lock().read_to_end(&mut data)?;
            (data, None)
        } else {
            let mut file = fs::File::open(path)?;
            let stat = file.metadata()?;
            #[cfg(unix)] {
                use std::os::unix::prelude::MetadataExt; // Ugh, this is so bad
                // +1 due to read_to_end's EOF check
                fallible_collections::FallibleVec::try_reserve(&mut data, stat.size() as usize + 1)?;
            }
            file.read_to_end(&mut data)?;
            (data, Some(stat))
        };
        self.load_data_with_stat(&data, stat)
    }

    #[inline(always)]
    pub fn load_data(&self, data: &[u8]) -> Result<Image, crate::Error> {
        self.load_data_with_stat(data, None)
    }

    fn load_data_with_stat(&self, data: &[u8], meta: Option<fs::Metadata>) -> Result<Image, crate::Error> {
        if data.starts_with(b"\x89PNG") {
            return self.load_png(data, meta);
        }

        #[cfg(feature = "avif")]
        if data.get(4..4+8) == Some(b"ftypavif") {
            return self.load_avif(data, meta).map_err(|_| crate::Error::new(28));
        }

        #[cfg(feature = "webp")]
        if data.get(0..4) == Some(b"RIFF") {
            return self.load_webp(data, meta).map_err(|_| crate::Error::new(28));
        }

        if data.get(0) == Some(&0xFF) {
            self.load_jpeg(data, meta)
        } else {
            Err(crate::Error::new(28))
        }
    }

    pub(crate) fn process_profile(&self, profile: LCMSResult<Profile>) -> Option<Profile> {
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
