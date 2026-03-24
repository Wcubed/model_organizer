use std::io::Cursor;

use glam::{Mat4, Vec3};
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

    for item in &model.build.items {
        if item.printable == Some(false) {
            // Item should not be printed. And thus not be shown.
            continue;
        }

        let Some(object) = model.resources.get_object(item.object_id) else {
            // Object does not exist, ignore.
            continue;
        };
        load_geometry(
            &object.geometry,
            item.transform,
            &model,
            &mut archive,
            &mut mesh,
        );
    }

    Ok(mesh)
}

fn load_geometry(
    geometry: &Geometry,
    transform: Mat4,
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
                let v1 = vertex_to_vector(mesh_3mf.vertices[triangle.v1 as usize], transform);
                let v2 = vertex_to_vector(mesh_3mf.vertices[triangle.v2 as usize], transform);
                let v3 = vertex_to_vector(mesh_3mf.vertices[triangle.v3 as usize], transform);

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
                let new_transform = transform * component.transform;

                match model.resources.get_object(component.object_id) {
                    Some(object) => {
                        load_geometry(&object.geometry, new_transform, model, archive, mesh);
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

                        // Sometimes a submodel contains multiple objects, so we need to make sure we select only the one we need.
                        if let Some(object) = submodel.resources.get_object(component.object_id) {
                            load_geometry(&object.geometry, new_transform, model, archive, mesh);
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

fn vertex_to_vector(vertex: Vertex, transform: Mat4) -> Vector3 {
    let point = Vec3::new(vertex.x, vertex.y, vertex.z);
    let projected = transform.project_point3(point);

    Vector3 {
        x: projected.x,
        y: projected.y,
        z: projected.z,
    }
}
