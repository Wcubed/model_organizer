use std::collections::BTreeSet;

use camino::{Utf8Path, Utf8PathBuf};
use godot::prelude::*;
use lexical_sort::natural_cmp;
use serde::{Deserialize, Serialize};

use crate::library_scanner::{CONFIG_FILE_NAME, PRINTABLE_EXTENSIONS, matches_extension};

#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct RustModel {
    /// Absolute path of this model folder.
    absolute_path: Utf8PathBuf,
    config: ModelConfig,

    printables: Vec<Printable>,
    misc_files: BTreeSet<Utf8PathBuf>,
}

#[godot_api]
impl RustModel {
    #[func]
    fn set_cover_image(&mut self, path: GString) {
        let new_path = Utf8PathBuf::from(&path.to_string());
        // TODO (2026-10-11): Load the new cover image, and only edit and save the config if the cover image could be loaded.
        self.config.cover_image_override = Some(new_path);
        self.save_config();
    }

    /// TODO (2026-10-11): Use enum.
    #[func]
    fn set_default_orientation(&mut self, orientation: u8) {
        // TODO (2026-10-11): Request re-rendering the printables.
        self.config.default_orientation = orientation;
        self.save_config();
    }
}

impl RustModel {
    pub fn new(absolute_path: &Utf8Path) -> Self {
        Self {
            absolute_path: absolute_path.to_path_buf(),
            config: ModelConfig::default(),
            printables: vec![],
            misc_files: BTreeSet::new(),
        }
    }

    /// Call this once after creating this resource.
    pub fn scan_model_dir(&mut self) {
        let root_path = self.absolute_path.clone();
        let config_path = root_path.join(CONFIG_FILE_NAME);
        self.config = ModelConfig::load_or_default(&config_path);

        self.printables.clear();
        self.misc_files.clear();

        self.scan_model_subdir(&root_path, &root_path);

        for printable in &mut self.printables {
            // See if there is a render of this printable.
            // TODO (2026-04-10): Encode the file extension in the render, so you can have an stl and 3mf with the same base name.
            // TODO (2026-04-10): Migrate to that new system.
            let render_path = printable.relative_path.with_extension("png");

            if self.misc_files.remove(&render_path) {
                // Render file is present.
                printable.rendered_image_path = Some(render_path);
            }
        }

        self.printables.sort();
    }

    fn scan_model_subdir(&mut self, dir: &Utf8Path, root_model_dir: &Utf8Path) {
        let Ok(read_dir) = dir.read_dir_utf8() else {
            return;
        };

        for entry in read_dir.filter_map(|entry| entry.ok()) {
            let path = entry.path().to_path_buf();
            if path.is_dir() {
                self.scan_model_subdir(&path, root_model_dir);
            } else {
                let Ok(relative_path) = path.strip_prefix(root_model_dir) else {
                    continue;
                };

                if matches_extension(relative_path, PRINTABLE_EXTENSIONS) {
                    // Printable file.
                    // Guard against broken thingyverse weirdness, where there are sometimes
                    // corrupted stl files in the images directory. Especially happens with files shared via torrents.
                    if relative_path
                        .parent()
                        .expect("We are in a dir, there is a parent")
                        .ends_with("images")
                    {
                        // Likely not printable.
                        continue;
                    }

                    if relative_path.ends_with(CONFIG_FILE_NAME) {
                        // Do not list the config file.
                        continue;
                    }

                    self.printables.push(Printable {
                        relative_path: relative_path.to_path_buf(),
                        rendered_image_path: None,
                    });
                } else {
                    // Not a printable file.
                    self.misc_files.insert(relative_path.to_path_buf());
                }
            }
        }
    }

    fn save_config(&self) {
        self.config.save(&self.absolute_path.join(CONFIG_FILE_NAME));
    }
}

#[derive(Debug, Eq)]
struct Printable {
    relative_path: Utf8PathBuf,
    rendered_image_path: Option<Utf8PathBuf>,
}

impl PartialEq for Printable {
    fn eq(&self, other: &Self) -> bool {
        self.relative_path == other.relative_path
    }
}

impl Ord for Printable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // The "natural" compare makes sure that "100" is sorted after, for example, "2"
        natural_cmp(self.relative_path.as_str(), other.relative_path.as_str())
    }
}

impl PartialOrd for Printable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Saved in a `.moa` file next to the model.
#[derive(Debug, Default, Serialize, Deserialize)]
struct ModelConfig {
    /// TODO (2026-10-11): Should be an enum, with Z_UP: 0, and Y_UP: 1.
    default_orientation: u8,
    cover_image_override: Option<Utf8PathBuf>,
}

impl ModelConfig {
    fn load_or_default(path: &Utf8Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|contents| {
                // When godot saved the preferences, there was a "[main]" prefixed. With toml this is not necessary, so we ignore it.
                let stripped_content = contents.strip_prefix("[main]").unwrap_or(&contents);
                toml::from_str(stripped_content).ok()
            })
            .unwrap_or_default()
    }

    fn save(&self, path: &Utf8Path) {
        let Ok(toml) = toml::to_string_pretty(self) else {
            godot_error!("Failed to convert config to toml");
            return;
        };
        let _ = std::fs::write(path, &toml)
            .inspect_err(|err| godot_error!("Failed to write config to file `{}`: {}", path, err));
    }
}
