use godot::{
    classes::{IPanelContainer, PanelContainer},
    prelude::*,
};

use crate::library::Library;

#[derive(GodotClass)]
#[class(init, base = PanelContainer)]
struct MainPanel {
    base: Base<PanelContainer>,
}

#[godot_api]
impl IPanelContainer for MainPanel {
    fn ready(&mut self) {
        // TODO (2026-10-14): Make sure image cache dir is created.
        // TODO (2026-10-14): Load settings.
        // TODO (2026-10-14): Create library struct based on settings.
        // TODO (2026-10-14): Start the library scan.
    }
}

// TODO (2026-10-14): Implement the rest of the main scene.
