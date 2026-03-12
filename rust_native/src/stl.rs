use godot::{
    classes::{ArrayMesh, FileAccess, mesh::PrimitiveType},
    global::Error,
    prelude::*,
};

use crate::ToCorruptError;

const HEADER_BYTES: usize = 80;
const BYTES_PER_TRIANGLE: usize = 50;

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

        let text_header = bytes
            .subarray(0..HEADER_BYTES)
            .get_string_from_ascii()
            .strip_edges(true, true);
        let result = if text_header.begins_with("solid") {
            load_stl_from_buffer_ascii(bytes)
        } else {
            load_stl_from_buffer_binary(bytes)
        };

        match result {
            Ok(mesh) => mesh.to_variant(),
            Err(error) => error.to_variant(),
        }
    }
}

enum ParseState {
    Header,
    Facet,
    OuterLoop,
    Vertex(usize),
    EndLoop,
    EndFacet,
    EndSolid,
}

/// Rust version of stl load function from stlio:
/// https://github.com/onze/godot-stl-io/blob/7700b430a2aac0bab1bee3dcad1ad70aa36e2017/addons/stl-io/importer.gd#L61
fn load_stl_from_buffer_ascii(bytes: PackedByteArray) -> Result<Gd<ArrayMesh>, Error> {
    let mut mesh = ArrayMesh::new_gd();
    let mut vertices = PackedVector3Array::new();
    let mut normals = PackedVector3Array::new();

    let stl_string = bytes.get_string_from_ascii().to_string();

    let mut state = ParseState::Header;
    let mut normal = Vector3::ZERO;
    let mut facet = [Vector3::ZERO; 3];

    for full_line in stl_string.lines() {
        let line = full_line.trim();

        match state {
            ParseState::Header => {
                state = ParseState::Facet;
            }
            ParseState::Facet => {
                if line.starts_with("endsolid") {
                    state = ParseState::EndSolid;
                    continue;
                }

                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.len() != 5 {
                    return Err(Error::ERR_FILE_CORRUPT);
                }
                // Line goes `facet normal <x> <y> <z>`
                normal = Vector3 {
                    x: tokens[2].parse::<f32>().corrupt_err()?,
                    y: tokens[3].parse::<f32>().corrupt_err()?,
                    z: tokens[4].parse::<f32>().corrupt_err()?,
                };

                normals.push(normal);
                normals.push(normal);
                normals.push(normal);

                state = ParseState::OuterLoop;
            }
            ParseState::OuterLoop => {
                state = ParseState::Vertex(0);
            }
            ParseState::Vertex(num) => {
                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.len() != 4 {
                    return Err(Error::ERR_FILE_CORRUPT);
                }
                // Line goes `vertex <x> <y> <z>`
                facet[num] = Vector3 {
                    x: tokens[1].parse::<f32>().corrupt_err()?,
                    y: tokens[2].parse::<f32>().corrupt_err()?,
                    z: tokens[3].parse::<f32>().corrupt_err()?,
                };

                if num >= facet.len() - 1 {
                    // Face complete.
                    let calculated_normal = (facet[0] - facet[2]).cross(facet[0] - facet[1]);

                    if calculated_normal.dot(normal) > 0.0 {
                        // Face is the right way up.
                        vertices.push(facet[0]);
                        vertices.push(facet[1]);
                        vertices.push(facet[2]);
                    } else {
                        // Face is upside down.
                        vertices.push(facet[2]);
                        vertices.push(facet[1]);
                        vertices.push(facet[0]);
                    }

                    state = ParseState::EndLoop;
                } else {
                    state = ParseState::Vertex(num + 1);
                }
            }
            ParseState::EndLoop => {
                state = ParseState::EndFacet;
            }
            ParseState::EndFacet => {
                state = ParseState::Facet;
            }
            ParseState::EndSolid => {
                // End of the stl.
                break;
            }
        }
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

/// Rust version of the stl load function from stlio:
/// https://github.com/onze/godot-stl-io/blob/7700b430a2aac0bab1bee3dcad1ad70aa36e2017/addons/stl-io/importer.gd#L139
fn load_stl_from_buffer_binary(bytes: PackedByteArray) -> Result<Gd<ArrayMesh>, Error> {
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
