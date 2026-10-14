use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use godot::prelude::*;
use lexical_sort::natural_cmp;

use crate::{
    model::RustModel,
    paths::{AbsolutePath, RelativePath},
};

pub const IGNORED_EXTENSIONS: &[&str] = &["zip", "rar", "moa", "md", "txt"];
pub const PRINTABLE_EXTENSIONS: &[&str] = &["stl", "obj", "3mf"];
pub const CONFIG_FILE_NAME: &str = "config.moa";

struct Library {
    library_dir: AbsolutePath,
    /// Contains the models after the scan is done.
    models: Vec<RustModel>,
    scan_thread: Option<JoinHandle<Vec<RustModel>>>,
}

impl Library {
    pub fn new(library_dir: AbsolutePath) -> Self {
        Self {
            library_dir,
            models: vec![],
            scan_thread: None,
        }
    }

    /// Starts a scan, if there isn't one ongoing.
    pub fn scan(&mut self) {
        if self.scan_thread.is_some() {
            // A scan is running, or the result has not yet been collected.
            return;
        }

        let dir = self.library_dir.clone();

        self.scan_thread = Some(thread::spawn(|| directory_scan_thread(dir)));
    }

    pub fn is_scanning(&self) -> bool {
        self.scan_thread.is_some()
    }

    /// Call this while [`Self::is_scanning`] is returns true.
    pub fn maybe_join_scan(&mut self) {
        if let Some(thread) = self.scan_thread.take_if(|thread| thread.is_finished()) {
            match thread.join() {
                Ok(models) => {
                    self.models = models;
                }
                Err(error) => {
                    godot_error!("Scan thread failed with: {:?}", error);
                    self.models.clear();
                }
            }
        }
    }
}

/// Scans the given directory for models. Then passes the model paths to the model scanning thread.
fn directory_scan_thread(library_dir: AbsolutePath) -> Vec<RustModel> {
    let (sender, receiver) = crossbeam::channel::unbounded();

    let model_thread = thread::spawn(move || model_scan_thread(receiver));

    scan_dir_for_models(&library_dir, &sender);
    // Notify the model threads that we are done.
    drop(sender);

    let mut models = model_thread.join().expect("Thread should not fail");
    models.sort_by(|a, b| natural_cmp(&a.name().to_lowercase(), &b.name().to_lowercase()));

    models
}

/// Recieves model paths from the directory scan thread, then scans the models.
fn model_scan_thread(model_receiver: Receiver<AbsolutePath>) -> Vec<RustModel> {
    let mut models = vec![];

    while let Ok(path) = model_receiver.recv() {
        let mut model = RustModel::new(path);
        model.scan_model_dir();
        models.push(model);
    }

    models
}

/// Scans the given directory recursively for models.
/// Adds any suspected model directories to the model queue.
fn scan_dir_for_models(dir: &AbsolutePath, model_sender: &Sender<AbsolutePath>) {
    let Ok(read_dir) = dir.read_dir() else {
        return;
    };

    let mut subdirs = vec![];

    for entry in read_dir.filter_map(|entry| entry.ok()) {
        let path = entry.path();

        if path.is_dir() {
            subdirs.push(path);
        } else {
            if matches_extension(&path, IGNORED_EXTENSIONS) {
                // Ignored file.
                continue;
            }

            // Directory is a model directory.
            model_sender
                .send(dir.clone())
                .expect("Receivers should be waiting for us to disconnect.");

            return;
        }
    }

    // No matching files. Check for "files" directory from thingyverse.
    if subdirs
        .iter()
        .find(|dir| dir.as_str().to_lowercase().ends_with("/files"))
        .is_some()
    {
        // Directory is a model directory.
        model_sender
            .send(dir.clone())
            .expect("Receivers should be waiting for us to disconnect.");
        return;
    }

    // Directory is not a model directory, scan subdirectories.
    for subdir in subdirs {
        scan_dir_for_models(&subdir, model_sender);
    }
}

/// Checks if the extension of the path is contained in the given list.
/// Expects the extensions list in lowercase, without the dot.
pub fn matches_extension(path: &AbsolutePath, extensions: &[&str]) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };

    extensions.contains(&ext.to_lowercase().as_str())
}
