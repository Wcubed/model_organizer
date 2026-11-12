use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use fuzzy_cmp::hybrid_compare;
use godot::{prelude::*, tools::get_autoload_by_name};
use lexical_sort::natural_cmp;

use crate::{
    model::RustModel,
    paths::{AbsolutePath, RelativePath},
};

pub const IGNORED_EXTENSIONS: &[&str] = &["zip", "rar", "moa", "md", "txt"];
pub const PRINTABLE_EXTENSIONS: &[&str] = &["stl", "obj", "3mf"];
pub const CONFIG_FILE_NAME: &str = "config.moa";

/// Library holding all the models.
/// Meant to be auto-loaded.
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct Library {
    base: Base<Node>,

    library_dir: Option<AbsolutePath>,
    /// Contains the models after the scan is done.
    models: Vec<Gd<RustModel>>,
    scan_thread: Option<JoinHandle<Vec<RustModel>>>,

    /// Result of a call to `search_models`.
    search_result: Vec<Gd<RustModel>>,
}

#[godot_api]
impl INode for Library {
    fn process(&mut self, _delta: f32) {
        // See if we are scanning, and update if so.
        self.maybe_join_scan();
    }
}

#[godot_api]
impl Library {
    #[signal]
    pub fn scan_started();

    #[signal]
    pub fn scan_complete();

    /// A call to `search_models` has completed.
    #[signal]
    pub fn model_search_complete();

    #[signal]
    pub fn library_dir_path_changed(new_path: String);
}

impl Library {
    pub fn singleton() -> Gd<Self> {
        get_autoload_by_name("LibraryGlobal")
    }

    pub fn get_library_dir(&self) -> &Option<AbsolutePath> {
        &self.library_dir
    }

    pub fn set_library_dir(&mut self, dir: AbsolutePath) {
        self.library_dir = Some(dir.clone());

        self.signals()
            .library_dir_path_changed()
            .emit(dir.as_str().to_string());
    }

    pub fn get_model_count(&self) -> usize {
        self.models.len()
    }

    /// Returns the models found by the call to `search_models`.
    pub fn get_search_result(&self) -> &[Gd<RustModel>] {
        &self.search_result
    }

    /// Search the model list and returns matches in order of best-to-worst.
    /// Is case-insensitive.
    /// If the search string is empty: returns the entire list.
    pub fn search_models(&mut self, search_str: &str) {
        if search_str.is_empty() {
            self.search_result = self.models.clone();
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
        self.search_result = scored.into_iter().map(|(_, model)| model).collect();
        self.signals().model_search_complete().emit();
    }

    /// Starts a scan, if there isn't one ongoing.
    pub fn scan(&mut self) {
        if self.scan_thread.is_some() {
            // A scan is running, or the result has not yet been collected.
            return;
        }
        let Some(library_dir) = self.library_dir.clone() else {
            return;
        };

        self.signals().scan_started().emit();

        self.scan_thread = Some(thread::spawn(|| directory_scan_thread(library_dir)));
    }

    pub fn is_scanning(&self) -> bool {
        self.scan_thread.is_some()
    }

    /// Call this while [`Self::is_scanning`] returns true.
    fn maybe_join_scan(&mut self) {
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

            self.signals().scan_complete().emit();
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
