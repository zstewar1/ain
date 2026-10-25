use std::path::{Path, PathBuf};

use crate::{AinStr, AinString};

impl AsRef<Path> for AinStr {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl AsRef<Path> for AinString {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl From<AinString> for PathBuf {
    fn from(value: AinString) -> Self {
        value.into_string().into()
    }
}
