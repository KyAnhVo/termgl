use crate::graphics::{
    mesh::Mesh,
    projection::Camera,
    vertex::{Material, Vertex},
};
use glam::{Vec3, Vec4, Vec4Swizzles};

/// Represents a point light source in the scene.
/// A point light source emits light uniformly in all directions from a single point.
///
/// Sometimes, we use the point light source for simplicity rather than it being a true point light source.
/// so we can have a "wrapper mesh" that is rendered in place of just the point light source.
pub struct PointLightSource {
    /// position of the light source in world space
    pub pos: Vec3,
    /// the mesh to render in place of the point light source
    pub wrapper_mesh: Mesh,
    /// intensity of diffuse term
    pub diffuse_intensity: Vec3,
    /// intensity of specular term
    pub specular_intensity: Vec3,
    /// intensity of ambient term
    pub ambient_intensity: Vec3,
}

impl PointLightSource {
    pub fn new(
        pos: Vec3,
        wrapper_mesh: Mesh,
        diffuse_intensity: Vec3,
        specular_intensity: Vec3,
        ambient_intensity: Vec3,
    ) -> Self {
        assert!(
            wrapper_mesh.no_shade == true,
            "wrapper mesh must have no_shade set to true"
        );
        Self {
            pos,
            wrapper_mesh,
            diffuse_intensity,
            specular_intensity,
            ambient_intensity,
        }
    }

    /// for Gouraud shading.
    /// These datas can be gathered simply from the Mesh object.
    pub fn shade_vertex(
        self,
        vertex: Vertex,
        material: Material,
        normal: Vec4,
        cam: Camera,
    ) -> Vec3 {
        // uses triangle.a as ref for most things, assume all 3 are equivalent
        let kd: Vec3 = vertex.color;
        let ks: Vec3 = material.ks;
        let ka: Vec3 = material.ka;
        let p: f32 = material.p;

        let n: Vec3 = normal.xyz();
        let v: Vec3 = cam.e.xyz() - vertex.pos.xyz();
        let l: Vec3 = self.pos - vertex.pos.xyz();
        let h_prenormalize: Vec3 = v + l;
        let r2: f32 = h_prenormalize.length_squared();
        let h = h_prenormalize.normalize();

        let ia: Vec3 = self.ambient_intensity;
        let id: Vec3 = self.diffuse_intensity;
        let is: Vec3 = self.specular_intensity;
        let ambient_term: Vec3 = ka * ia;
        let diffuse_term: Vec3 = kd * (id / r2) * (n.dot(l)).max(0.0);
        let specular_term: Vec3 = ks * (is / r2) * (n.dot(h)).max(0.0).powf(p);

        ambient_term + diffuse_term + specular_term
    }

    /// Shades a vertex using the Phong shading model.
    /// This requires interpolated normal, world position and color values calculated prior.
    pub fn phong_shade(
        self,
        pos: Vec3,
        normal: Vec4,
        material: Material,
        color: Vec3,
        cam: Camera,
    ) -> Vec3 {
        let kd: Vec3 = color;
        let ks: Vec3 = material.ks;
        let ka: Vec3 = material.ka;
        let p: f32 = material.p;

        let n: Vec3 = normal.xyz();
        let v: Vec3 = cam.e.xyz() - pos;
        let l: Vec3 = self.pos - pos;
        let h_prenormalize: Vec3 = v + l;
        let r2: f32 = h_prenormalize.length_squared();
        let h = h_prenormalize.normalize();

        let ia: Vec3 = self.ambient_intensity;
        let id: Vec3 = self.diffuse_intensity;
        let is: Vec3 = self.specular_intensity;
        let ambient_term: Vec3 = ka * ia;
        let diffuse_term: Vec3 = kd * (id / r2) * (n.dot(l)).max(0.0);
        let specular_term: Vec3 = ks * (is / r2) * (n.dot(h)).max(0.0).powf(p);

        ambient_term + diffuse_term + specular_term
    }

    /// Shade a vertex in the light source's wrapper mesh.
    /// This uses only the normal and the camera's position.
    pub fn shade_light_vertex(
        self,
        pos: Vec3,
        normal: Vec4,
        material: Material,
        color: Vec3,
        cam: Camera,
    ) -> Vec3 {
    }
}
