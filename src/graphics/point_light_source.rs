use crate::graphics::{
    camera::Camera, mesh::Mesh, options::LightSourceShadingMode, vertex::Material,
};
use glam::{Vec3, Vec4, Vec4Swizzles};

/// Represents a point light source in the scene.
/// A point light source emits light uniformly in all directions from a single point.
///
/// Sometimes, we use the point light source for simplicity rather than it being a true point light source.
/// so we can have a "wrapper mesh" that is rendered in place of just the point light source.
#[derive(Clone)]
pub struct PointLightSource {
    /// position of the light source in world space
    pub pos: Vec3,
    /// the mesh to render in place of the point light source
    pub wrapper_mesh: Option<Mesh>,
    /// intensity of diffuse term (to other objects)
    pub diffuse_intensity: Vec3,
    /// intensity of specular term (to other objects)
    pub specular_intensity: Vec3,
    /// intensity of ambient term (to other objects)
    pub ambient_intensity: Vec3,
    /// the light source mesh's shining constant
    /// matters when shading the light source itself
    pub shining_constant: Vec3,
    /// the shading mode to use when shading the light source itself
    pub shading_mode: LightSourceShadingMode,
}

impl PointLightSource {
    pub fn new(
        pos: Vec3,
        wrapper_mesh: Option<Mesh>,
        diffuse_intensity: Vec3,
        specular_intensity: Vec3,
        ambient_intensity: Vec3,
        shining_constant: Vec3,
        shading_mode: LightSourceShadingMode,
    ) -> Self {
        assert!(
            match &wrapper_mesh {
                Some(mesh) => !mesh.no_shade,
                None => true,
            },
            "wrapper mesh must have no_shade set to true"
        );
        Self {
            pos,
            wrapper_mesh,
            diffuse_intensity,
            specular_intensity,
            ambient_intensity,
            shining_constant,
            shading_mode,
        }
    }

    /// Shades a point given position, normal, material, and original color
    pub fn shade(
        &self,
        pos: Vec3,
        normal: Vec4,
        material: Material,
        color: Vec3,
        cam: &Camera,
    ) -> Vec3 {
        let kd: Vec3 = color;
        let ks: Vec3 = material.specular_constant;
        let ka: Vec3 = color * material.ambient_constant;
        let p: f32 = material.specular_exponent;

        let n: Vec3 = normal.xyz();
        let v: Vec3 = (cam.pos.xyz() - pos).normalize();
        let l_prenormalized: Vec3 = self.pos - pos;
        let l: Vec3 = l_prenormalized.normalize();
        let r2: f32 = l_prenormalized.length_squared();
        let h = (v + l).normalize();

        let ia: Vec3 = self.ambient_intensity;
        let id: Vec3 = self.diffuse_intensity;
        let is: Vec3 = self.specular_intensity;

        let ambient_term: Vec3 = ka * ia;
        let diffuse_term: Vec3 = kd * (id / r2) * n.dot(l).max(0.0);
        let specular_term: Vec3 = ks * (is / r2) * n.dot(h).max(0.0).powf(p);

        let color: Vec3 = ambient_term + diffuse_term + specular_term;
        Vec3::new(
            color.x.clamp(0.0, 1.0),
            color.y.clamp(0.0, 1.0),
            color.z.clamp(0.0, 1.0),
        )
    }

    pub fn scale_intensity(&mut self, diffuse_scale: f32, specular_scale: f32, ambient_scale: f32) {
        assert!(diffuse_scale >= 0.0 && specular_scale >= 0.0 && ambient_scale >= 0.0);
        self.diffuse_intensity *= diffuse_scale;
        self.specular_intensity *= specular_scale;
        self.ambient_intensity *= ambient_scale;
    }
}
