use godot::{
    classes::{IPanelContainer, PanelContainer},
    prelude::*,
};

use crate::{connect_signals, library::Library};

#[derive(GodotClass)]
#[class(init, base = PanelContainer)]
struct ScanProgressBar {
    base: Base<PanelContainer>,
}

#[godot_api]
impl IPanelContainer for ScanProgressBar {
    fn ready(&mut self) {
        let lib = Library::singleton();
        connect_signals!(self, [
            lib, scan_started => Self::on_scan_started;
            lib, scan_complete => Self::on_scan_complete;
        ]);
    }
}

impl ScanProgressBar {
    fn on_scan_started(&mut self) {
        self.base_mut().show();
    }

    fn on_scan_complete(&mut self) {
        self.base_mut().hide();
    }
}
