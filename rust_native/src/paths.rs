use std::{
    io,
    path::{Path, StripPrefixError},
};

use camino::{ReadDirUtf8, Utf8DirEntry, Utf8Path, Utf8PathBuf};
use lexical_sort::natural_cmp;

/// Guaranteed to be an absolute path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AbsolutePath(Utf8PathBuf);

impl AbsolutePath {
    /// Returns a new path if it is absolute.
    pub fn new(path: &Utf8Path) -> Result<Self, ()> {
        if path.is_absolute() {
            Ok(Self(path.to_path_buf()))
        } else {
            Err(())
        }
    }

    pub fn inner(&self) -> &Utf8Path {
        &self.0
    }

    pub fn join(&self, relative: &RelativePath) -> Self {
        Self(self.0.join(&relative.0))
    }

    pub fn join_str(&self, relative: &str) -> Self {
        Self(self.0.join(relative))
    }

    pub fn read_dir(&self) -> Result<ReadDirAbsolute, std::io::Error> {
        self.0.read_dir_utf8().map(ReadDirAbsolute)
    }

    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name()
    }

    pub fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    pub fn strip_prefix(&self, base: &Self) -> Result<RelativePath, StripPrefixError> {
        self.0
            .strip_prefix(&base.0)
            .map(|relative| RelativePath(relative.to_path_buf()))
    }

    pub fn extension(&self) -> Option<&str> {
        self.0.extension()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Guaranteed to be a relative path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelativePath(Utf8PathBuf);

impl RelativePath {
    /// Returns a new path if it is relative.
    pub fn new(path: &Utf8Path) -> Result<Self, ()> {
        if !path.is_absolute() {
            Ok(Self(path.to_path_buf()))
        } else {
            Err(())
        }
    }

    pub fn natural_cmp(&self, other: &Self) -> std::cmp::Ordering {
        // The "natural" compare makes sure that "100" is sorted after, for example, "2"
        natural_cmp(self.0.as_str(), other.0.as_str())
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(|parent| Self(parent.to_path_buf()))
    }

    pub fn ends_with(&self, child: impl AsRef<Path>) -> bool {
        self.0.ends_with(child)
    }

    pub fn with_extension(&self, arg: &str) -> Self {
        Self(self.0.with_extension(arg))
    }

    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct ReadDirAbsolute(ReadDirUtf8);

impl Iterator for ReadDirAbsolute {
    type Item = io::Result<AbsoluteDirEntry>;

    fn next(&mut self) -> Option<io::Result<AbsoluteDirEntry>> {
        self.0.next().map(|entry| entry.map(AbsoluteDirEntry))
    }
}

pub struct AbsoluteDirEntry(Utf8DirEntry);

impl AbsoluteDirEntry {
    pub fn path(&self) -> AbsolutePath {
        AbsolutePath(self.0.path().to_path_buf())
    }
}
