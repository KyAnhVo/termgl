use glam::{Mat3, Mat4, Vec3};
use termgl::graphics::{Material, Mesh};

use rand::random_range;
use std::f32::consts::PI;

struct Planet {
    /// Name of planet (e.g. Earth, Venus, etc.)
    pub name: String,

    /// Radius (of the sphere representing it)
    pub rad: f32,

    /// essentially dist from sun
    pub orbit_rad: f32,

    /// rotational speed (dTheta/dt)
    pub rotational_velocity: f32,

    /// orbital speed (dTheta/dt)
    pub orbital_velocity: f32,

    /// original location
    pub location_original: Vec3,

    /// original orientation of the xyz space
    pub orientation_original: Mat3,

    /// A planet is either a planet or a sun... yes very unintuitive
    pub is_sun: bool,

    /// the actual mesh of the planet
    pub mesh: Mesh,
}

impl Planet {
    pub fn new(
        name: String,
        rad: f32,
        orbit_rad: f32,
        rotational_velocity: f32,
        orbital_velocity: f32,
        is_sun: bool,
    ) -> Self {
        let random_num: f32 = random_range(0..100) as f32 / 100.0;
        let default_rot: Mat3 = Mat3::from_rotation_z(2.0 * PI * random_num);
        let original_pos: Vec3 = default_rot * Vec3::ONE * rad;
        let original_orientation: Mat3 = default_rot * Mat3::IDENTITY;

        let material: Material = Material::new(Vec3::ONE, Vec3::ONE * 0.1, 200.0);
        let mut mesh: Mesh = Mesh::new(original_pos, original_orientation, material, is_sun);
        mesh.add_texture_map(&format!("assets/{}.jpg", name));

        Self {
            name,
            rad,
            orbit_rad,
            rotational_velocity,
            orbital_velocity,
            location_original: original_pos,
            orientation_original: original_orientation,
            is_sun,
            mesh,
        }
    }

    pub fn move_planet(&mut self, t: f32, t_scale: f32) {
        let rot: Mat3 = Mat3::from_rotation_z(t * t_scale * self.rotational_velocity);
        let orbit: Mat3 = Mat3::from_rotation_z(t * t_scale * self.orbital_velocity);
        self.mesh.move_origin_to(orbit * self.location_original);
        self.mesh.rotate(rot);
    }
}
