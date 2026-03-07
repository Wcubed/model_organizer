use std::io::Read;

use godot::{classes::file_access::ModeFlags, prelude::*};
use tinystl::StlData;

struct RustNativeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustNativeExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct StlLoader {}

#[godot_api]
impl IRefCounted for StlLoader {
    fn init(_base: Base<RefCounted>) -> Self {
        Self {}
    }
}

#[godot_api]
impl StlLoader {
    #[func]
    fn load_from_file(path: GString) {
        let Ok(file) = GFile::open(&path, ModeFlags::READ) else {
            godot_print!("File open error");
            return;
        };

        let result = StlData::read_buffer(file);

        match result {
            Ok(stl) => {
                godot_print!("Success!");
            }
            Err(error) => {
                godot_print!("{}", error);
            }
        }
    }
}
