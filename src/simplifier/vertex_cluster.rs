use crate::graphics::{Material, Mesh, Vertex, VertexIndices};
use glam::{Vec3, Vec4};
use std::f32;

/// Implements the vertex cluster simplification algorithm by Rossignac and Borrel.
pub fn vertex_cluster(mesh: &Mesh, (dx, dy, dz): (f32, f32, f32)) -> Mesh {
    let mut simplified_mesh = Mesh::new(mesh.material, mesh.no_shade);
    simplified_mesh.default_color = mesh.default_color;

    // TODO: Implement vertex cluster simplification algorithm

    simplified_mesh
}
