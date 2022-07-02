use ash::vk::{
    api_version_major, api_version_minor, api_version_patch, api_version_variant, make_api_version,
};

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy)]
pub struct ApiVersion {
    variant: u8,
    major: u8,
    minor: u8,
    patch: u16,
}

impl ApiVersion {
    pub fn new(variant: u8, major: u8, minor: u8, patch: u16) -> Self {
        ApiVersion {
            variant,
            major,
            minor,
            patch,
        }
    }

    pub fn u32_patchless(&self) -> u32 {
        make_api_version(self.variant as u32, self.major as u32, self.minor as u32, 0)
    }

    pub fn u32(&self) -> u32 {
        make_api_version(
            self.variant as u32,
            self.major as u32,
            self.minor as u32,
            self.patch as u32,
        )
    }
}

impl From<u32> for ApiVersion {
    fn from(version: u32) -> Self {
        ApiVersion {
            variant: api_version_variant(version) as u8,
            major: api_version_major(version) as u8,
            minor: api_version_minor(version) as u8,
            patch: api_version_patch(version) as u16,
        }
    }
}

impl From<ApiVersion> for u32 {
    fn from(version: ApiVersion) -> Self {
        make_api_version(
            version.variant as u32,
            version.major as u32,
            version.minor as u32,
            version.patch as u32,
        )
    }
}

impl Ord for ApiVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.major > other.major {
            return std::cmp::Ordering::Greater;
        } else if self.major < other.major {
            return std::cmp::Ordering::Less;
        }

        if self.minor > other.minor {
            return std::cmp::Ordering::Greater;
        } else if self.minor < other.minor {
            return std::cmp::Ordering::Less;
        }

        if self.patch > other.patch {
            return std::cmp::Ordering::Greater;
        } else if self.patch < other.patch {
            return std::cmp::Ordering::Less;
        }

        std::cmp::Ordering::Equal
    }
}
