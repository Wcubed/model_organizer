use crossbeam::channel::Receiver;
use godot::{
    classes::{IPanelContainer, Label, PanelContainer, TextureProgressBar},
    prelude::*,
};

use crate::{
    connect_signals,
    library::{Library, ScanProgress},
};

#[derive(GodotClass)]
#[class(init, base = PanelContainer)]
struct ScanProgressBar {
    base: Base<PanelContainer>,
    progress_receiver: Option<Receiver<ScanProgress>>,

    models_found: u32,
    models_scanned: u32,

    #[init(node = "%ProgressBar")]
    progress_bar: OnReady<Gd<TextureProgressBar>>,
    #[init(node = "%AmountLabel")]
    label: OnReady<Gd<Label>>,
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

    fn process(&mut self, _delta: f32) {
        let Some(receiver) = &self.progress_receiver else {
            return;
        };

        while let Ok(progress) = receiver.try_recv() {
            match progress {
                ScanProgress::FoundModel => self.models_found += 1,
                ScanProgress::ScannedModel => self.models_scanned += 1,
            }
        }

        let percentage = self.models_scanned as f64 / self.models_found.max(1) as f64;
        self.progress_bar.set_value(percentage);
        self.label
            .set_text(&format!("{}/{}", self.models_scanned, self.models_found).to_gstring());
    }
}

impl ScanProgressBar {
    fn on_scan_started(&mut self) {
        self.base_mut().show();
        self.progress_receiver = Library::singleton().bind().get_scan_progress();

        self.models_found = 0;
        self.models_scanned = 0;

        self.progress_bar.set_value(0.0);
        self.label.set_text("0/0");
    }

    fn on_scan_complete(&mut self) {
        self.base_mut().hide();
        self.progress_receiver = None;
    }
}
