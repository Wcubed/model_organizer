use camino::Utf8PathBuf;
use godot::{
    classes::{
        Button, FileDialog, IPanelContainer, Label, LineEdit, PanelContainer, object::ConnectFlags,
    },
    prelude::*,
};

use crate::{connect_signals, library::Library, paths::AbsolutePath};

#[derive(GodotClass)]
#[class(init, base = PanelContainer)]
struct Toolbar {
    base: Base<PanelContainer>,

    #[init(node = "%PathButton")]
    path_button: OnReady<Gd<Button>>,
    #[init(node = "%ReloadButton")]
    reload_button: OnReady<Gd<Button>>,
    #[init(node = "%ClearSearchButton")]
    clear_button: OnReady<Gd<Button>>,
    #[init(node = "%SearchEdit")]
    search_edit: OnReady<Gd<LineEdit>>,
    #[init(node = "%AmountLabel")]
    amount_label: OnReady<Gd<Label>>,

    #[init(node = "FolderDialog")]
    folder_dialog: OnReady<Gd<FileDialog>>,
}

#[godot_api]
impl IPanelContainer for Toolbar {
    fn ready(&mut self) {
        self.amount_label.set_text("");

        Library::singleton()
            .signals()
            .model_search_complete()
            .builder()
            .flags(ConnectFlags::DEFERRED)
            .connect_other_mut(self, Self::on_search_complete);

        connect_signals!(self, [
            Library::singleton(), scan_complete => Self::on_scan_complete;
            self.path_button, pressed => Self::on_path_button_pressed;
            self.reload_button, pressed => Self::on_reload_button_pressed;
            self.clear_button, pressed => Self::on_clear_button_pressed;
            self.search_edit, text_submitted => Self::on_search_edit_submitted;
            self.folder_dialog, dir_selected => Self::on_dir_selected;
        ]);
    }
}

impl Toolbar {
    fn run_search(&self) {
        Library::singleton()
            .bind_mut()
            .search_models(&self.search_edit.get_text().to_string());
    }

    fn on_scan_complete(&mut self) {
        self.run_search();
    }

    fn on_search_complete(&mut self) {
        let library = Library::singleton();
        self.amount_label.set_text(&format!(
            "{}/{}",
            library.bind().get_search_result().len(),
            library.bind().get_model_count()
        ));
    }

    fn on_path_button_pressed(&mut self) {
        if let Some(path) = Library::singleton().bind().get_library_dir() {
            self.folder_dialog.set_current_path(path.as_str());
        }

        self.folder_dialog.show();
    }

    fn on_reload_button_pressed(&mut self) {
        Library::singleton().bind_mut().scan();
    }

    fn on_dir_selected(&mut self, dir_path: GString) {
        let utf_path = Utf8PathBuf::from(dir_path.to_string());
        if let Ok(absolute_path) = AbsolutePath::new(&utf_path) {
            let mut lib = Library::singleton();
            let mut lib_mut = lib.bind_mut();
            lib_mut.set_library_dir(absolute_path);
            lib_mut.scan();
        }
    }

    fn on_clear_button_pressed(&mut self) {
        self.search_edit.set_text("");
        self.run_search();
    }

    fn on_search_edit_submitted(&mut self, _new_text: GString) {
        self.run_search();
    }
}
