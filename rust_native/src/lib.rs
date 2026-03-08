use godot::{
    classes::{
        ArrayMesh,
        file_access::ModeFlags,
        mesh::{ArrayType, PrimitiveType},
    },
    obj::IndexEnum,
    prelude::*,
};

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
        let Ok(mut file) = GFile::open(&path, ModeFlags::READ) else {
            godot_print!("File open error");
            return 0.to_variant();
        };

        let result: Result<stl_io::IndexedMesh, _> = stl_io::read_stl(&mut file);

        godot_print!("Loaded");

        match result {
            Ok(stl) => {
                // Each face has 3 vertices, so we know the capacity needed.
                let mut vertex_array = Vec::with_capacity(stl.vertices.len());
                let mut index_array = Vec::with_capacity(stl.faces.len() * 3);

                for vertex in stl.vertices {
                    vertex_array.push(Vector3::new(vertex[0], vertex[1], vertex[2]));
                }
                for face in stl.faces {
                    for index in face.vertices {
                        index_array.push(index as i32);
                    }
                }

                let packed_vertices = PackedVector3Array::from(vertex_array);
                let packed_indices = PackedInt32Array::from(index_array);

                let arrays = Array::from(&[
                    packed_vertices.to_variant(), // Vertices
                    Variant::nil(),               // Normals
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
                    packed_indices.to_variant(), // Indices
                ]);

                let mut output = ArrayMesh::new_gd();
                godot_print!("go: {}", output.get_surface_count());
                output.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);

                godot_print!("{}", output.get_surface_count());

                output.to_variant()
            }
            Err(_error) => 0.to_variant(),
        }
    }
}
