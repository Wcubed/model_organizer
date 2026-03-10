use godot::{
    classes::{ArrayMesh, FileAccess, mesh::PrimitiveType},
    global::Error,
    prelude::*,
};

const HEADER_BYTES: usize = 80;
const BYTES_PER_TRIANGLE: usize = 50;

struct RustNativeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustNativeExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct StlLoader {}

#[godot_api]
impl IRefCounted for StlLoader {
    fn init(_base: Base<RefCounted>) -> Self {
        Self {}
    }
}

#[godot_api]
impl StlLoader {
    /// Returns ArrayMesh or an integer to indicate error.
    #[func]
    fn load_from_file(path: GString) -> Variant {
        let bytes = FileAccess::get_file_as_bytes(&path);
        if bytes.is_empty() {
            return FileAccess::get_open_error().to_variant();
        }

        // TODO (2026-03-10): Load ascii stl files.

        match load_stl_from_buffer(bytes) {
            Ok(mesh) => mesh.to_variant(),
            Err(error) => error.to_variant(),
        }
    }
}

/// Rust version of the stl load function from stlio:
/// https://github.com/onze/godot-stl-io/blob/7700b430a2aac0bab1bee3dcad1ad70aa36e2017/addons/stl-io/importer.gd#L139
fn load_stl_from_buffer(bytes: PackedByteArray) -> Result<Gd<ArrayMesh>, Error> {
    let mut mesh = ArrayMesh::new_gd();
    // Header usually not important, so we skip it.
    let mut offset = HEADER_BYTES;

    let triangle_count = bytes.decode_u32(offset).corrupt_err()? as usize;
    offset += 4;

    // Create the vectors with known capacity, to reduce re-allocation.
    let mut vertices = PackedVector3Array::new();
    vertices.resize(triangle_count * 3);
    let mut normals = PackedVector3Array::new();
    normals.resize(triangle_count * 3);

    if offset + triangle_count * BYTES_PER_TRIANGLE > bytes.len() {
        return Err(Error::ERR_FILE_CORRUPT);
    }

    for index in 0..triangle_count {
        let normal = get_vector(&bytes, &mut offset)?;
        normals[index * 3] = normal;
        normals[index * 3 + 1] = normal;
        normals[index * 3 + 2] = normal;

        let v1 = get_vector(&bytes, &mut offset)?;
        let v2 = get_vector(&bytes, &mut offset)?;
        let v3 = get_vector(&bytes, &mut offset)?;

        let calculated_normal = (v1 - v3).cross(v1 - v2);

        if calculated_normal.dot(normal) > 0.0 {
            // Face is the right way up.
            vertices[index * 3] = v1;
            vertices[index * 3 + 1] = v2;
            vertices[index * 3 + 2] = v3;
        } else {
            // Face is upside down.
            vertices[index * 3] = v3;
            vertices[index * 3 + 1] = v2;
            vertices[index * 3 + 2] = v1;
        }

        // According to wikipedia, most software does not use attributes, and neither do we.
        let _attribute = bytes.decode_u16(offset).corrupt_err()?;
        offset += 2;
    }

    let arrays = Array::from(&[
        vertices.to_variant(), // Vertices
        normals.to_variant(),  // Normals
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(),
        Variant::nil(), // Indices
    ]);
    mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);

    Ok(mesh)
}

fn get_vector(bytes: &PackedByteArray, offset: &mut usize) -> Result<Vector3, Error> {
    let vector = Vector3::new(
        bytes.decode_float(*offset).corrupt_err()?,
        bytes.decode_float(*offset + 4).corrupt_err()?,
        bytes.decode_float(*offset + 8).corrupt_err()?,
    );
    *offset += 12;
    Ok(vector)
}

trait ToCorruptError<T> {
    fn corrupt_err(self) -> Result<T, Error>;
}

impl<T> ToCorruptError<T> for Result<T, ()> {
    fn corrupt_err(self) -> Result<T, Error> {
        self.map_err(|()| Error::ERR_FILE_CORRUPT)
    }
}
