use camino::Utf8Path;
use godot::{
    classes::{ConfigFile, IPanelContainer, PanelContainer},
    global::Error,
    prelude::*,
};

use crate::{connect_signals, library::Library, paths::AbsolutePath};

const CONFIG_FILE_NAME: &str = "user://config.cfg";

#[derive(GodotClass)]
#[class(init, base = PanelContainer)]
struct MainPanel {
    base: Base<PanelContainer>,
}

#[godot_api]
impl IPanelContainer for MainPanel {
    fn ready(&mut self) {
        connect_signals!(self, [
            Library::singleton(), library_dir_path_changed => Self::save_settings;
        ]);

        // TODO (2026-10-14): Make sure image cache dir is created.
        // TODO (2026-10-14): Load settings.
        // TODO (2026-10-14): Create library struct based on settings.
        // TODO (2026-10-14): Start the library scan.
        self.load_settings();
        Library::singleton().bind_mut().scan();
    }
}

impl MainPanel {
    fn load_settings(&self) {
        let mut config = ConfigFile::new_gd();
        let err = config.load(CONFIG_FILE_NAME);
        if err != Error::OK {
            return;
        }

        let library_dir_variant = config
            .get_value_ex("main", "library_dir")
            .default(&GString::new().to_variant())
            .done();
        if let Some(library_dir) = library_dir_variant
            .try_to::<GString>()
            .map(|dir| dir.to_string())
            .ok()
            .and_then(|dir| AbsolutePath::new(Utf8Path::new(&dir)).ok())
        {
            Library::singleton().bind_mut().set_library_dir(library_dir);
        }
    }

    fn save_settings(&mut self, new_dir: String) {
        let mut config = ConfigFile::new_gd();
        config.set_value("main", "library_dir", &new_dir.to_gstring().to_variant());

        config.save(CONFIG_FILE_NAME);
    }
}

// TODO (2026-10-14): Implement the rest of the main scene.
