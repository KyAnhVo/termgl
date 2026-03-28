use crate::graphics::mesh::Mesh;
use glam::Vec3;

/// Represents a mesh light source in the scene.
/// MeshLightSource is a mesh that emits light in the scene.
/// (e.g. the sun, a light bulb, a screen)
///
/// Shading with this light source implements the area light model
/// for each triangle in the mesh.
/// Note: this is very costly to render, so prefer using a point light source instead.
pub struct MeshLightSource {
    pub lighting_object: Mesh,
    pub diffuse_intensity: Vec3,
    pub specular_intensity: Vec3,
    pub ambient_intensity: Vec3,
}
