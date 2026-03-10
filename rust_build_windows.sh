# Shell script so godot can cross-compile rust for windows on linux.
cd rust_native

TARGET="x86_64-pc-windows-gnu"

cross build --target ${TARGET} --release