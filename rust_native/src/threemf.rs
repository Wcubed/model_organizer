use std::{io::Cursor, iter::Zip};

use godot::{
    classes::{ArrayMesh, FileAccess, mesh::PrimitiveType},
    global::Error,
    prelude::*,
};
use lib3mf_core::{
    Geometry, Lib3mfError, Model, Vertex,
    archive::{ArchiveReader, ZipArchiver, find_model_path},
    parser::parse_model,
};

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct ThreemfLoader {}

#[godot_api]
impl IRefCounted for ThreemfLoader {
    fn init(_base: Base<RefCounted>) -> Self {
        Self {}
    }
}

#[godot_api]
impl ThreemfLoader {
    /// Returns ArrayMesh or an integer to indicate error.
    #[func]
    fn load_from_file(path: GString) -> Variant {
        godot_print!("Loading: {}", path);
        let bytes = FileAccess::get_file_as_bytes(&path);
        if bytes.is_empty() {
            return FileAccess::get_open_error().to_variant();
        }

        match load_3mf_from_buffer(bytes) {
            Ok(mesh) => mesh.to_variant(),
            Err(error) => {
                godot_print!("Failed to load 3mf: {}", error);
                Error::ERR_FILE_CORRUPT.to_variant()
            }
        }
    }
}

fn load_3mf_from_buffer(bytes: PackedByteArray) -> Result<Gd<ArrayMesh>, Lib3mfError> {
    let mut archive = ZipArchiver::new(Cursor::new(bytes.as_slice()))?;

    let model_path = find_model_path(&mut archive)?;
    let model_data = archive.read_entry(&model_path)?;
    let model = parse_model(Cursor::new(model_data))?;

    let mut mesh = ArrayMesh::new_gd();

    for object in model.resources.iter_objects() {
        godot_print!("{:?}", object.id);
    }

    for item in &model.build.items {
        let Some(object) = model.resources.get_object(item.object_id) else {
            // Object does not exist, ignore.
            continue;
        };
        // TODO (2026-03-12): Use transform?
        load_geometry(&object.geometry, &model, &mut archive, &mut mesh);
    }

    godot_print!("{}", mesh.get_surface_count());

    Ok(mesh)
}

fn load_geometry(
    geometry: &Geometry,
    model: &Model,
    archive: &mut dyn ArchiveReader,
    mesh: &mut Gd<ArrayMesh>,
) {
    match geometry {
        Geometry::Mesh(mesh_3mf) => {
            // Create the vectors with known capacity, to reduce re-allocation.
            let mut vertices = PackedVector3Array::new();
            vertices.resize(mesh_3mf.triangles.len() * 3);
            let mut normals = PackedVector3Array::new();
            normals.resize(mesh_3mf.triangles.len() * 3);

            for (index, triangle) in mesh_3mf.triangles.iter().enumerate() {
                let v1 = vertex_to_vector(mesh_3mf.vertices[triangle.v1 as usize]);
                let v2 = vertex_to_vector(mesh_3mf.vertices[triangle.v2 as usize]);
                let v3 = vertex_to_vector(mesh_3mf.vertices[triangle.v3 as usize]);

                let calculated_normal = (v1 - v3).cross(v1 - v2);

                normals[index * 3] = calculated_normal;
                normals[index * 3 + 1] = calculated_normal;
                normals[index * 3 + 2] = calculated_normal;

                vertices[index * 3] = v1;
                vertices[index * 3 + 1] = v2;
                vertices[index * 3 + 2] = v3;
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
        }
        Geometry::Components(components) => {
            for component in &components.components {
                match model.resources.get_object(component.object_id) {
                    Some(object) => {
                        // TODO (2026-03-12): Use transform?
                        load_geometry(&object.geometry, model, archive, mesh);
                    }
                    None => {
                        // Can't find the object by ID. Try to find by path.
                        let Some(path) = &component.path else {
                            continue;
                        };
                        let Ok(submodel_bytes) = archive.read_entry(path) else {
                            continue;
                        };

                        let Ok(submodel) = parse_model(Cursor::new(submodel_bytes)) else {
                            continue;
                        };

                        // With the submodel we don't need to look at the "Build" we can immediately iter the objects.
                        for object in submodel.resources.iter_objects() {
                            // TODO (2026-03-12): Use transform?
                            load_geometry(&object.geometry, model, archive, mesh);
                        }
                    }
                }
            }
        }
        _ => {
            // Rest we don't know what to do with.
        }
    }
}

fn vertex_to_vector(vertex: Vertex) -> Vector3 {
    Vector3 {
        x: vertex.x,
        y: vertex.y,
        z: vertex.z,
    }
}
