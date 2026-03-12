use std::num::ParseFloatError;

use godot::{global::Error, prelude::*};

mod stl;
mod threemf;

struct RustNativeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustNativeExtension {}

trait ToCorruptError<T> {
    fn corrupt_err(self) -> Result<T, Error>;
}

impl<T> ToCorruptError<T> for Result<T, ()> {
    fn corrupt_err(self) -> Result<T, Error> {
        self.map_err(|()| Error::ERR_FILE_CORRUPT)
    }
}

impl<T> ToCorruptError<T> for Result<T, ParseFloatError> {
    fn corrupt_err(self) -> Result<T, Error> {
        self.map_err(|_| Error::ERR_FILE_CORRUPT)
    }
}
