use glam::{Vec3, Vec4Swizzles, Vec4};
use crate::graphics::{projection::Camera, triangle::{Material, Mesh, Vertex}};

pub struct PointLightSource {
    pub pos: Vec3,
    pub diffuse_intensity: Vec3,    // intensity of diffuse term
    pub specular_intensity: Vec3,   // intensity for specular term
    pub ambient_intensity: Vec3,    // intensity for ambient terms
}

impl PointLightSource {
    pub fn new(pos: Vec3, diffuse_intensity: Vec3, specular_intensity: Vec3, ambient_intensity: Vec3) -> Self {
        Self {
            pos,
            diffuse_intensity,
            specular_intensity,
            ambient_intensity,
        }
    }

    /// gouraud shade vertex
    pub fn shade_vertex(self, vertex: Vertex, material: Material, normal: Vec4, cam: Camera) -> Vec3 {
        // uses triangle.a as ref for most things, assume all 3 are equivalent
        let kd: Vec3 = vertex.color;
        let ks: Vec3 = material.ks;
        let ka: Vec3 = material.ka;
        let p: f32   = material.p;

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
}
