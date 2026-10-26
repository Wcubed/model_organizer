use std::num::ParseFloatError;

use godot::{global::Error, prelude::*};

mod library;
mod model;
mod paths;
mod stl;
mod threemf;
mod ui;

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

/// Convenience macro for connecting signals to a function on another object.
/// For example `self`.
#[macro_export]
macro_rules! connect_signals {
    ($target:expr, [$($source:expr, $signal:ident => $function:expr;)*]) => {{
        $(
            $source.signals()
                .$signal()
                .connect_other($target, $function);
        )*

    }};
}
