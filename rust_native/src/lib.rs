use godot::{
    classes::{ArrayMesh, file_access::ModeFlags, mesh::PrimitiveType},
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
            return 0.to_variant();
        };

        let result: Result<stl_io::IndexedMesh, _> = stl_io::read_stl(&mut file);

        match result {
            Ok(stl) => {
                // Each face has 3 vertices, so we know the capacities needed.
                let mut vertex_array = Vec::with_capacity(stl.vertices.len());
                let mut normal_array = vec![Vector3::ZERO; stl.vertices.len()];
                let mut index_array = Vec::with_capacity(stl.faces.len() * 3);

                for vertex in stl.vertices {
                    vertex_array.push(Vector3::new(vertex[0], vertex[1], vertex[2]));
                }
                for face in stl.faces {
                    // Godots triangle vertex order is aparently inverse from that of stl_io.
                    // Hence the reversing of the iterator.
                    for index in face.vertices.iter().rev() {
                        index_array.push(*index as i32);
                        normal_array[*index] +=
                            Vector3::new(face.normal[0], face.normal[1], face.normal[2]);
                    }
                }

                // Normalize the normals.
                for normal in &mut normal_array {
                    *normal = normal.normalized();
                }

                let packed_vertices = PackedVector3Array::from(vertex_array);
                let packed_normals = PackedVector3Array::from(normal_array);
                let packed_indices = PackedInt32Array::from(index_array);

                let arrays = Array::from(&[
                    packed_vertices.to_variant(), // Vertices
                    packed_normals.to_variant(),  // Normals
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

                output.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
                output.to_variant()
            }
            Err(_error) => 0.to_variant(),
        }
    }
}
