use camino::{Utf8Path, Utf8PathBuf};
use godot::{
    classes::{Button, FileDialog, IPanelContainer, PanelContainer},
    prelude::*,
};

use crate::{library::Library, paths::AbsolutePath};

#[derive(GodotClass)]
#[class(init, base = PanelContainer)]
struct Toolbar {
    base: Base<PanelContainer>,

    #[init(node = "%PathButton")]
    path_button: OnReady<Gd<Button>>,
    #[init(node = "FolderDialog")]
    folder_dialog: OnReady<Gd<FileDialog>>,
}

#[godot_api]
impl IPanelContainer for Toolbar {
    fn ready(&mut self) {
        self.path_button
            .signals()
            .pressed()
            .connect_other(self, Self::on_path_button_pressed);

        self.folder_dialog
            .signals()
            .dir_selected()
            .connect_other(self, Self::on_dir_selected);
    }
}

impl Toolbar {
    fn on_path_button_pressed(&mut self) {
        if let Some(path) = Library::singleton().bind().get_library_dir() {
            self.folder_dialog.set_current_path(path.as_str());
        }

        self.folder_dialog.show();
    }

    fn on_dir_selected(&mut self, dir_path: GString) {
        let utf_path = Utf8PathBuf::from(dir_path.to_string());
        if let Ok(absolute_path) = AbsolutePath::new(&utf_path) {
            Library::singleton().bind_mut().scan(absolute_path);
        }
    }
}
