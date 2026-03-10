# Shell script so godot can start the rust build.
set -euo pipefail

cd rust_native
cargo build --release