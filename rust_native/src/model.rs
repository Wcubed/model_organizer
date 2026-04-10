use std::collections::HashSet;

use camino::{Utf8Path, Utf8PathBuf};
use godot::prelude::*;

use crate::library_scanner::{CONFIG_FILE_NAME, PRINTABLE_EXTENSIONS, matches_extension};

#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct RustModel {
    /// Absolute path of this model folder.
    absolute_path: Utf8PathBuf,

    printables: Vec<Printable>,
    misc_files: HashSet<Utf8PathBuf>,
}

impl RustModel {
    pub fn new(absolute_path: &Utf8Path) -> Self {
        Self {
            absolute_path: absolute_path.to_path_buf(),
            printables: vec![],
            misc_files: HashSet::new(),
        }
    }

    /// Call this after creating a new Model, or if the folder has changed.
    pub fn scan_model_dir(&mut self) {
        // TODO (2026-04-10): Load configs.
        let path = self.absolute_path.clone();

        self.printables.clear();
        self.misc_files.clear();

        self.scan_model_subdir(&path);

        for printable in &mut self.printables {
            // See if there is a render of this printable.
            // TODO (2026-04-10): Encode the file extension in the render, so you can have an stl and 3mf with the same base name.
            // TODO (2026-04-10): Migrate to that new system.
            let render_path = printable.absolute_path.with_extension("png");

            if self.misc_files.remove(&render_path) {
                // Render file is present.
                printable.rendered_image_path = Some(render_path);
            }
        }
    }

    fn scan_model_subdir(&mut self, dir: &Utf8Path) {
        let Ok(read_dir) = dir.read_dir_utf8() else {
            return;
        };

        for entry in read_dir.filter_map(|entry| entry.ok()) {
            let path = entry.path().to_path_buf();
            if path.is_dir() {
                self.scan_model_subdir(&path);
            } else {
                if matches_extension(&path, PRINTABLE_EXTENSIONS) {
                    // Printable file.
                    // Guard against broken thingyverse weirdness, where there are sometimes
                    // corrupted stl files in the images directory. Especially happens with files shared via torrents.
                    if path
                        .parent()
                        .expect("We are in a dir, there is a parent")
                        .ends_with("images")
                    {
                        // Likely not printable.
                        continue;
                    }

                    if path.ends_with(CONFIG_FILE_NAME) {
                        // Do not list the config file.
                        continue;
                    }

                    self.printables.push(Printable {
                        absolute_path: path,
                        rendered_image_path: None,
                    });
                } else {
                    // Not a printable file.
                    self.misc_files.insert(path);
                }
            }
        }
    }
}

struct Printable {
    absolute_path: Utf8PathBuf,
    rendered_image_path: Option<Utf8PathBuf>,
}
