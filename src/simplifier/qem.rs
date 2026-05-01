use crate::graphics::{Material, Mesh, Vertex, VertexIndices};
use glam::{Vec3, Vec4};
use std::f32;

pub fn qem(mesh: &Mesh) -> Mesh {
    let mut simplified_mesh = Mesh::new(mesh.material, mesh.no_shade);
    simplified_mesh.default_color = mesh.default_color;

    // TODO: Implement QEM algorithm

    simplified_mesh
}
