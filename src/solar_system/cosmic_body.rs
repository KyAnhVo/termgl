use glam::{Mat3, Vec3};

use crate::graphics::{mesh::Mesh, vertex::Material};

pub struct CosmicBody {
    pub mesh: Mesh,

    /// the radius of the body from its center
    pub radius: f32,

    /// the distance of the body from the sun's center
    pub orbital_radius: f32,

    /// the color of the body (suppose they are all the same first)
    pub color: Vec3,

    /// the material properties of the body
    pub material: Material,

    /// the rotation speed of the body around the sun
    pub orbital_speed: f32,

    /// the time that has elapsed since the body was created
    pub elapsed_time: f32,
}

impl CosmicBody {
    pub fn new(
        radius: f32,
        orbital_radius: f32,
        color: Vec3,
        material: Material,
        orbital_speed: f32,
        original_time: f32,
        longtitudes: usize,
        latitudes: usize,
    ) -> Self {
        let mut mesh: Mesh = Mesh::new(orbital_radius * Vec3::X, Mat3::IDENTITY, material, false);
        mesh.create_sphere(longtitudes, latitudes, radius, color);

        Self {
            mesh,
            radius,
            orbital_radius,
            color,
            material,
            orbital_speed,
            elapsed_time: original_time,
        }
    }

    /// Orbits the body around the sun, updating its position based on the elapsed time and orbital speed.
    pub fn orbit(&mut self, delta_t: f32) {
        self.elapsed_time += delta_t;
        let new_origin: Vec3 = Mat3::from_rotation_y(self.orbital_speed * self.elapsed_time)
            * (Vec3::X * self.orbital_radius);
        self.mesh.move_origin_to(new_origin);
    }
}
