use crate::graphics::mesh::Mesh;
use crate::graphics::point_light_source::PointLightSource;
use crate::graphics::projection::Camera;
use crate::graphics::vertex::{Material, Vertex};

use glam::{Vec3, Vec4};

pub struct Shader {
    pub point_light_sources: Vec<PointLightSource>,
    // pub mesh_light_sources: Vec<MeshLightSource>,
}

impl Shader {
    pub fn new() -> Self {
        Self {
            point_light_sources: Vec::new(),
            // mesh_light_sources: Vec::new(),
        }
    }

    pub fn add_point_light_source(&mut self, light: PointLightSource) {
        self.point_light_sources.push(light);
    }

    pub fn shade_point_phong(
        &self,
        pos: Vec3,
        normal: Vec4,
        material: Material,
        color: Vec3,
        cam: Camera,
    ) -> Vec3 {
        let mut final_color: Vec3 = Vec3::ZERO;
        for light in &self.point_light_sources {
            final_color += light.shade(pos, normal, material, color, cam);
        }
        final_color
    }

    /// used for Gouraud shading.
    pub fn shade_vertex(
        &self,
        vertex: Vertex,
        normal: Vec4,
        material: Material,
        cam: Camera,
    ) -> Vec3 {
        let mut final_color = Vec3::ZERO;
        for light in &self.point_light_sources {
            final_color += light.shade_vertex(vertex, material, normal, cam);
        }
        final_color
    }

    /// shade vertices of a mesh in-place,
    /// only touches the projected VAO so
    /// the function expects mesh.projected_vao
    /// to be filled.
    pub fn shade_mesh_gouraud(&self, mesh: &mut Mesh, cam: Camera) {
        if mesh.no_shade {
            return;
        }
        for v in 0..mesh.vertices.len() {
            mesh.raster_vertices[v].color = self.shade_vertex(
                mesh.vertices_world_space[v],
                mesh.normals_world_space[v],
                mesh.material,
                cam,
            );
        }
    }
}
