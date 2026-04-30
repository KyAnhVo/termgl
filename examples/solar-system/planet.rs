use glam::{Mat3, Vec3};
use termgl::graphics::{Material, Mesh};

use std::rc::Rc;

use rand::random_range;
use std::f32::consts::PI;

pub struct Planet {
    /// Name of planet (e.g. Earth, Venus, etc.)
    pub name: Rc<str>,

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

    /// the ring mesh, cause saturn you know...
    pub ring_mesh: Option<Mesh>,
}

impl Planet {
    pub fn new(
        name: Rc<str>,
        rad: f32,
        orbit_rad: f32,
        rotational_velocity: f32,
        orbital_velocity: f32,
        is_sun: bool,
    ) -> Self {
        let random_num: f32 = random_range(0..100) as f32 / 100.0;
        let default_rot: Mat3 = Mat3::from_rotation_y(2.0 * PI * random_num);
        let original_pos: Vec3 = default_rot * Vec3::X * orbit_rad;
        let original_orientation: Mat3 = default_rot * Mat3::IDENTITY;

        let material: Material = Material::new(Vec3::ONE, 0.1, 200.0);
        let mut mesh: Mesh = Mesh::create_sphere(rad, original_pos, material, Vec3::ONE, 16, 16);
        if is_sun {
            mesh.no_shade = true;
        }
        mesh.no_shade = true;
        mesh.add_texture_map(&format!("examples/assets/{}.jpg", name));

        let saturn_ring: Option<Mesh> = if &*name == "saturn" {
            let mut ring: Mesh = Mesh::create_ring(
                rad,
                rad * 2.0,
                original_pos,
                material,
                Vec3::ONE,
                2.0 * PI / 64.0,
            );
            ring.add_texture_map("examples/assets/saturn_ring.jpg");
            Some(ring)
        } else {
            None
        };

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
            ring_mesh: saturn_ring,
        }
    }

    pub fn move_planet(&mut self, dt: f32, t: f32, t_scale: f32) {
        let rot: Mat3 = Mat3::from_rotation_y(dt * t_scale * self.rotational_velocity);
        let orbit: Mat3 = Mat3::from_rotation_y(t * t_scale * self.orbital_velocity);
        self.mesh.move_origin_to(orbit * self.location_original);
        self.mesh.rotate(rot);
        self.mesh.finalize_mesh();
        match &mut self.ring_mesh {
            Some(mesh) => {
                mesh.move_origin_to(orbit * self.location_original);
                mesh.rotate(rot);
                mesh.finalize_mesh();
            }
            None => {}
        }
    }
}
