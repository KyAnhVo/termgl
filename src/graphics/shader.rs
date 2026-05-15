use crate::graphics::camera::Camera;
use crate::graphics::point_light_source::PointLightSource;
use crate::graphics::vertex::Material;

use glam::{Vec3, Vec4};

/// Accumulates light sources and evaluates shading at a surface point.
pub struct Shader {
    pub point_light_sources: Vec<PointLightSource>,
}

impl Shader {
    /// Creates an empty shader with no light sources.
    pub fn new() -> Self {
        Self {
            point_light_sources: Vec::new(),
            // mesh_light_sources: Vec::new(),
        }
    }

    /// Registers a point light source to be included in shading calculations.
    pub fn add_point_light_source(&mut self, light: PointLightSource) {
        self.point_light_sources.push(light);
    }

    pub(crate) fn shade_point_phong(
        &self,
        pos: Vec3,
        normal: Vec4,
        material: Material,
        color: Vec3,
        cam: &Camera,
    ) -> Vec3 {
        let mut final_color: Vec3 = Vec3::ZERO;
        for light in &self.point_light_sources {
            final_color += light.shade(pos, normal, material, color, cam);
        }
        final_color
    }
}
