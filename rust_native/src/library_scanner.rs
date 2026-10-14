use core::error;
use std::thread::{self, JoinHandle};

use camino::{Utf8Path, Utf8PathBuf};
use crossbeam::channel::{Receiver, Sender};
use godot::prelude::*;
use lexical_sort::natural_cmp;

use crate::model::RustModel;

pub const IGNORED_EXTENSIONS: &[&str] = &["zip", "rar", "moa", "md", "txt"];
pub const PRINTABLE_EXTENSIONS: &[&str] = &["stl", "obj", "3mf"];
pub const CONFIG_FILE_NAME: &str = "config.moa";

struct LibraryScanner {
    library_dir: Utf8PathBuf,
    scan_thread: Option<JoinHandle<Vec<RustModel>>>,
}

impl LibraryScanner {
    pub fn new(library_dir: GString) -> Self {
        let dir_string = library_dir.to_string();
        let dir = Utf8PathBuf::from(&dir_string);

        Self {
            library_dir: dir,
            scan_thread: None,
        }
    }

    /// Starts a scan, if there isn't one ongoing.
    pub fn scan(&mut self, library_dir: GString) {
        if self.scan_thread.is_some() {
            // A scan is running, or the result has not yet been collected.
            return;
        }

        self.scan_thread = Some(thread::spawn(|| directory_scan_thread(library_dir)));
    }

    pub fn is_scanning(&self) -> bool {
        self.scan_thread.is_some()
    }

    /// Will return the result of the scan 1 time after the scan is complete.
    pub fn get_result(&mut self) -> Option<Vec<RustModel>> {
        if let Some(thread) = self.scan_thread.take_if(|thread| thread.is_finished()) {
            match thread.join() {
                Ok(models) => Some(models),
                Err(error) => {
                    godot_error!("Scan thread failed with: {:?}", error);
                    Some(vec![])
                }
            }
        } else {
            None
        }
    }
}

/// Scans the given directory for models. Then passes the model paths to the model scanning thread.
fn directory_scan_thread(library_dir: GString) -> Vec<RustModel> {
    let dir_string = library_dir.to_string();
    let dir = Utf8Path::new(&dir_string);

    let (sender, receiver) = crossbeam::channel::unbounded();

    let model_thread = thread::spawn(move || model_scan_thread(receiver));

    scan_dir_for_models(dir, &sender);
    // Notify the model threads that we are done.
    drop(sender);

    let mut models = model_thread.join().expect("Thread should not fail");
    models.sort_by(|a, b| natural_cmp(&a.name().to_lowercase(), &b.name().to_lowercase()));

    models
}

/// Recieves model paths from the directory scan thread, then scans the models.
fn model_scan_thread(model_receiver: Receiver<Utf8PathBuf>) -> Vec<RustModel> {
    let mut models = vec![];

    while let Ok(path) = model_receiver.recv() {
        let mut model = RustModel::new(&path);
        model.scan_model_dir();
        models.push(model);
    }

    models
}

/// Scans the given directory recursively for models.
/// Adds any suspected model directories to the model queue.
fn scan_dir_for_models(dir: &Utf8Path, model_sender: &Sender<Utf8PathBuf>) {
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

    // No matching files. Check for "files" directory from thingyverse.
    if subdirs
        .iter()
        .find(|dir| dir.as_str().to_lowercase().ends_with("/files"))
        .is_some()
    {
        // Directory is a model directory.
        model_sender
            .send(dir.to_path_buf())
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
pub fn matches_extension(path: &Utf8Path, extensions: &[&str]) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };

    extensions.contains(&ext.to_lowercase().as_str())
}
