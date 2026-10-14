use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use godot::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    library::{CONFIG_FILE_NAME, PRINTABLE_EXTENSIONS, matches_extension},
    paths::{AbsolutePath, RelativePath},
};

// TODO (2026-10-11): Implement all the missing stuff from the godot version of this class.
// TODO (2026-10-11): After implementing the missing stuff: rename this one to "model" and refactor the inevatable problems in godot.
#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct RustModel {
    name: String,
    path: AbsolutePath,
    config: ModelConfig,

    printables: Vec<Printable>,
    misc_files: BTreeSet<RelativePath>,
}

#[godot_api]
impl RustModel {
    #[func]
    fn set_cover_image_override(&mut self, path: GString) {
        let new_path = Utf8PathBuf::from(&path.to_string());
        // TODO (2026-10-11): Load the new cover image, and only edit and save the config if the cover image could be loaded.
        self.config.cover_image_override = Some(new_path);
        self.save_config();
    }

    #[func]
    fn set_default_orientation(&mut self, orientation: RenderSideUp) {
        // TODO (2026-10-11): Request re-rendering the printables.
        self.config.default_orientation = orientation;
        self.save_config();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &AbsolutePath {
        &self.path
    }
}

impl RustModel {
    pub fn new(path: AbsolutePath) -> Self {
        // The name of the model is the directory name (yes you get it by calling 'file_name').
        let name = path.file_name().unwrap_or("no name").to_string();

        Self {
            name,
            path,
            config: ModelConfig::default(),
            printables: vec![],
            misc_files: BTreeSet::new(),
        }
    }

    /// Call this once after creating this resource.
    pub fn scan_model_dir(&mut self) {
        let root_path = self.path.clone();
        let config_path = root_path.join_str(CONFIG_FILE_NAME);
        self.config = ModelConfig::load_or_default(&config_path);

        self.printables.clear();
        self.misc_files.clear();

        self.scan_model_subdir(&root_path, &root_path);

        for printable in &mut self.printables {
            // See if there is a render of this printable.
            // TODO (2026-04-10): Encode the file extension in the render, so you can have an stl and 3mf with the same base name.
            // TODO (2026-04-10): Migrate to that new system.
            let render_path = printable.path.with_extension("png");

            if self.misc_files.remove(&render_path) {
                // Render file is present.
                printable.rendered_image_path = Some(render_path);
            }
        }

        self.printables.sort();
    }

    fn scan_model_subdir(&mut self, dir: &AbsolutePath, root_model_dir: &AbsolutePath) {
        let Ok(read_dir) = dir.read_dir() else {
            return;
        };

        for entry in read_dir.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.is_dir() {
                self.scan_model_subdir(&path, root_model_dir);
            } else {
                let Ok(relative_path) = path.strip_prefix(root_model_dir) else {
                    continue;
                };

                if matches_extension(&path, PRINTABLE_EXTENSIONS) {
                    // Printable file.
                    // Guard against broken thingyverse weirdness, where there are sometimes
                    // corrupted stl files in the images directory. Especially happens with files shared via torrents.
                    if relative_path
                        .parent()
                        .map(|parent| parent.ends_with("images"))
                        == Some(true)
                    {
                        // Likely not printable.
                        continue;
                    }

                    if relative_path.ends_with(CONFIG_FILE_NAME) {
                        // Do not list the config file.
                        continue;
                    }

                    self.printables.push(Printable {
                        path: relative_path,
                        rendered_image_path: None,
                    });
                } else {
                    // Not a printable file.
                    self.misc_files.insert(relative_path);
                }
            }
        }
    }

    fn save_config(&self) {
        self.config.save(&self.path.join_str(CONFIG_FILE_NAME));
    }
}

#[derive(Debug, Eq)]
struct Printable {
    path: RelativePath,
    rendered_image_path: Option<RelativePath>,
}

impl PartialEq for Printable {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Ord for Printable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.natural_cmp(&other.path)
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
    default_orientation: RenderSideUp,
    cover_image_override: Option<Utf8PathBuf>,
}

impl ModelConfig {
    fn load_or_default(path: &AbsolutePath) -> Self {
        std::fs::read_to_string(path.inner())
            .ok()
            .and_then(|contents| {
                // Change from the old format to the new.
                let replaced_contents = &contents
                    .replace(
                        "[main]\ndefault_orientation=0",
                        "default_orientation = \"Z\"",
                    )
                    .replace(
                        "[main]\ndefault_orientation=1",
                        "default_orientation = \"Y\"",
                    );
                toml::from_str(replaced_contents).ok()
            })
            .unwrap_or_default()
    }

    fn save(&self, path: &AbsolutePath) {
        let Ok(toml) = toml::to_string_pretty(self) else {
            godot_error!("Failed to convert config to toml");
            return;
        };
        let _ = std::fs::write(path.inner(), &toml).inspect_err(|err| {
            godot_error!("Failed to write config to file `{}`: {}", path.inner(), err)
        });
    }
}

#[derive(Debug, Default, Clone, Copy, GodotConvert, Var, Export, Serialize, Deserialize)]
#[godot(via = GString)]
pub enum RenderSideUp {
    #[default]
    Z,
    Y,
    X,
}
