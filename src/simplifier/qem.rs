use crate::graphics::{Material, Mesh, Vertex, VertexIndices};
use glam::{Vec3, Vec4};
use std::f32;

/// Implements the Quadratic Error Metrics (QEM) simplification algorithm by Garland and Heckbert.
pub fn qem(mesh: &Mesh) -> Mesh {
    let mut simplified_mesh = Mesh::new(mesh.material, mesh.no_shade);
    simplified_mesh.default_color = mesh.default_color;

    // TODO: Implement QEM algorithm

    simplified_mesh
}
