use std::thread::{self, JoinHandle, scope};

use crossbeam::channel::{Receiver, Sender};
use fuzzy_cmp::hybrid_compare;
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
    models: Vec<Gd<RustModel>>,
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

    /// Search the model list and returns matches in order of best-to-worst.
    /// Is case-insensitive.
    /// If the search string is empty: returns the entire list.
    pub fn search_models(&self, search_str: &str) -> Vec<Gd<RustModel>> {
        if search_str.is_empty() {
            return self.models.clone();
        }

        let lowercase_search = search_str.to_lowercase();

        let mut scored: Vec<(f32, Gd<RustModel>)> = self
            .models
            .iter()
            .cloned()
            .map(|model| {
                let score = {
                    hybrid_compare(
                        &lowercase_search,
                        &model.bind().path().as_str().to_lowercase(),
                    )
                };

                (score, model)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        // TODO (2026-10-14): Also search the printable files, and add that somehow to the score?

        scored.sort_by(|(score, _), (other, _)| {
            score
                .partial_cmp(other)
                .expect("We know these floats cannot be NaN")
        });

        // TODO (2026-10-14): There is probably a better way than allocating 2 vectors. But this works.
        scored.into_iter().map(|(_, model)| model).collect()
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

    /// Call this while [`Self::is_scanning`] returns true.
    pub fn maybe_join_scan(&mut self) {
        if let Some(thread) = self.scan_thread.take_if(|thread| thread.is_finished()) {
            match thread.join() {
                Ok(models) => {
                    self.models = models.into_iter().map(|model| model.to_gd()).collect();
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

    let library_root = library_dir.clone();
    let model_thread = thread::spawn(move || model_scan_thread(receiver, library_root));

    scan_dir_for_models(&library_dir, &library_dir, &sender);
    // Notify the model threads that we are done.
    drop(sender);

    let mut models = model_thread.join().expect("Thread should not fail");
    models.sort_by(|a, b| natural_cmp(&a.name().to_lowercase(), &b.name().to_lowercase()));

    models
}

/// Recieves model paths from the directory scan thread, then scans the models.
fn model_scan_thread(
    model_receiver: Receiver<RelativePath>,
    library_root: AbsolutePath,
) -> Vec<RustModel> {
    let mut models = vec![];

    while let Ok(path) = model_receiver.recv() {
        let mut model = RustModel::new(path, library_root.clone());
        model.scan_model_dir();
        models.push(model);
    }

    models
}

/// Scans the given directory recursively for models.
/// Adds any suspected model directories to the model queue.
fn scan_dir_for_models(
    dir: &AbsolutePath,
    library_root: &AbsolutePath,
    model_sender: &Sender<RelativePath>,
) {
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
            if let Ok(relative_dir) = dir.strip_prefix(library_root) {
                model_sender
                    .send(relative_dir)
                    .expect("Receivers should be waiting for us to disconnect.");
            }

            return;
        }
    }

    // No matching files. Check for "files" directory from thingyverse.
    if subdirs
        .iter()
        .find(|dir| dir.as_str().to_lowercase().ends_with("/files"))
        .is_some()
    {
        if let Ok(relative_dir) = dir.strip_prefix(library_root) {
            model_sender
                .send(relative_dir)
                .expect("Receivers should be waiting for us to disconnect.");
        }
        return;
    }

    // Directory is not a model directory, scan subdirectories.
    for subdir in subdirs {
        scan_dir_for_models(&subdir, library_root, model_sender);
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
