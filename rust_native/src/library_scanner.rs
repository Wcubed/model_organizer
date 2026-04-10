use std::{sync::Arc, thread};

use camino::{Utf8Path, Utf8PathBuf};
use crossbeam::{
    channel::{Receiver, Sender},
    queue::SegQueue,
};
use godot::prelude::*;

use crate::model::RustModel;

pub const IGNORED_EXTENSIONS: &[&str] = &["zip", "rar", "moa"];
pub const PRINTABLE_EXTENSIONS: &[&str] = &["stl", "obj", "3mf"];
pub const CONFIG_FILE_NAME: &str = "config.moa";

/// Make a new scanner for each scan.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct LibraryScanner {}

#[godot_api]
impl IRefCounted for LibraryScanner {
    fn init(_base: Base<RefCounted>) -> Self {
        Self {}
    }
}

#[godot_api]
impl LibraryScanner {
    #[func]
    fn scan(&mut self, library_dir: GString) {
        let dir_string = library_dir.to_string();
        let dir = Utf8Path::new(&dir_string);

        let (sender, receiver) = crossbeam::channel::unbounded();

        let model_thread = thread::spawn(move || model_scan_thread(receiver));

        scan_dir(dir, &sender);
        // Notify the model threads that we are done.
        drop(sender);

        let models = model_thread.join().expect("Thread should not fail");
        godot_print!("processed: {}", models.len());
    }
}

/// Scans the given directory recursively for models.
/// Adds any suspected model directories to the model queue.
fn scan_dir(dir: &Utf8Path, model_sender: &Sender<Utf8PathBuf>) {
    let Ok(read_dir) = dir.read_dir_utf8() else {
        return;
    };

    let mut subdirs = vec![];

    for entry in read_dir.filter_map(|entry| entry.ok()) {
        let path = entry.path().to_path_buf();

        if path.is_dir() {
            subdirs.push(path);
        } else {
            if matches_extension(&path, IGNORED_EXTENSIONS) {
                // Ignored file.
                continue;
            }

            // Directory is a model directory.
            model_sender
                .send(dir.to_path_buf())
                .expect("Receivers should be waiting for us to disconnect.");

            return;
        }
    }

    // Directory is not a model directory, scan subdirectories.
    for subdir in subdirs {
        scan_dir(&subdir, model_sender);
    }
}

/// Checks if the extension of the path is contained in the given list.
/// Expects the extensions list in lowercase, without the dot.
pub fn matches_extension(path: &Utf8Path, extensions: &[&str]) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };

    extensions.contains(&ext.to_lowercase().as_str())
}

fn model_scan_thread(model_receiver: Receiver<Utf8PathBuf>) -> Vec<RustModel> {
    let mut models = vec![];

    while let Ok(path) = model_receiver.recv() {
        let mut model = RustModel::new(&path);
        model.scan_model_dir();
        models.push(model);
    }

    models
}
