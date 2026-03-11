# Shell script so godot can cross-compile rust for windows on linux.
cd rust_native

TARGET="x86_64-pc-windows-gnu"

cargo build --target ${TARGET} --release 2>&1 | tee target/windows_build.log