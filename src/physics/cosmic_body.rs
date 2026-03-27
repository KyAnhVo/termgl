use std::f32;
use std::f32::consts::PI;

use rand::{RngExt};
use glam::{Mat3, Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::graphics::{vertex::{Material, Vertex}};

pub struct CosmicBody {
    pub original_pos: Vec3,

    /// position wrt the sun
    pub pos: Vec3,

    /// velocity of rotation around the sun (rad/day)
    pub orbital_angular_velocity: f32,

    /// days per orbit
    pub days_per_orbit: u32,

    /// normal of the plane containing the rotate path
    pub rotational_normal: Vec3,

    /// color of the cosmic body, rgb
    pub color: Vec3,

    /// radius to draw the cosmic body
    pub radius: f32,
}

impl CosmicBody {
    const MAT: Material = Material { ks: Vec3::ZERO, ka: Vec3 {x: 2.0, y: 2.0, z: 2.0}, p: 3.0 };

    pub fn rot_x(theta: f32) -> Mat3 {
        let (sin, cos): (f32, f32) = (theta.sin(), theta.cos());
        Mat3::from_cols(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, cos, sin),
            Vec3::new(0.0, -sin, cos)
        )
    }

    pub fn rot_y(theta: f32) -> Mat3 {
        let (sin, cos): (f32, f32) = (theta.sin(), theta.cos());
        Mat3::from_cols(
            Vec3::new(cos, 0.0, -sin),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(sin, 0.0, cos),
        )
    }

    pub fn rot_z(theta: f32) -> Mat3 {
        let (sin, cos): (f32, f32) = (theta.sin(), theta.cos());
        Mat3::from_cols(
            Vec3::new(cos, sin, 0.0),
            Vec3::new(-sin, cos, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        )
    }

    pub fn new(
        original_pos: Vec3, 
        days_per_orbit: u32,
        rotational_normal: Vec3, 
        color: Vec3, 
        radius: f32
    ) -> Self {
        let orbital_angular_velocity: f32 = if days_per_orbit == 0 { 
            0.0
        } else { 
            2.0 * f32::consts::PI / (days_per_orbit as f32) 
        };

        Self {
            original_pos,
            pos: original_pos,
            orbital_angular_velocity: orbital_angular_velocity.sqrt() / 10.0,
            days_per_orbit,
            rotational_normal: rotational_normal.normalize(),
            color,
            radius,
        }
    }

    pub fn orbit(&mut self, days_passed: u32) {
        if self.orbital_angular_velocity.abs() < f32::EPSILON {
            return;
        }
        let rad: f32 = self.orbital_angular_velocity * days_passed as f32;
        let r: Mat3 = if self.rotational_normal == Vec3::Z {
            Mat3::from_cols(
                Vec3::new(rad.cos(),    rad.sin(),  0.0),
                Vec3::new(-rad.sin(),   rad.cos(),  0.0),
                Vec3::new(0.0,          0.0,        1.0),
            )
        } else {
            let nnt: Mat3 = Mat3::from_cols(
                self.rotational_normal * self.rotational_normal.x,
                self.rotational_normal * self.rotational_normal.y,
                self.rotational_normal * self.rotational_normal.z
            );
            let n_dual: Mat3 = Mat3::from_cols(
                Vec3::new(0.0,                          self.rotational_normal.z,   -self.rotational_normal.y),
                Vec3::new(-self.rotational_normal.z,    0.0,                        self.rotational_normal.x),
                Vec3::new(self.rotational_normal.y,     -self.rotational_normal.x,  0.0),
            );
            rad.cos() * Mat3::IDENTITY 
            + (1.0 - rad.cos()) * nnt
            + rad.sin() * n_dual
        };

        self.pos = r * self.original_pos;
    }

    pub fn to_triangles(&mut self, latitudes: usize, longtitudes: usize) -> Vec<(Vertex, Vertex, Vertex)> {
        let mut prime_meridian: Vec<Vec3> = vec![];
        let null_island: Vec3 = Vec3::X * self.radius;

        // div 2 mult 2 plus 1 to get the "floor even", then add 1 for center.
        let latitudes: usize = latitudes / 2 * 2 + 1;

        for latitude in 0..latitudes {
            let theta: f32 = PI * (-0.5 + latitude as f32 / (latitudes - 1) as f32);
            let rot: Mat3 = Self::rot_z(theta);
            prime_meridian.push(rot * null_island);
        }

        let mut vertices: Vec<Vec<Vertex>> = vec![vec![]; latitudes / 2 * 2 + 1];
        for longtitude in 0..=longtitudes {

        }

        vec![]
    }
}

pub struct CosmicSimulator {
    pub planets: Vec<CosmicBody>,
    pub sun: CosmicBody,
    pub days_passed: Vec<u32>,
    pub orbit_triangles: Vec<(Vertex, Vertex, Vertex)>,
}

impl CosmicSimulator {
    // ORIGINAL DATA (DISTANCES AND RING COUNT ARE REAL)
    //
    // Planet,     Distance from Sun (Avg. AU),    Orbital Period (Earth Days),    Radius (km),    Surface Color (Approx. RGB),    Has Ring (Count)
    // Mercury,    0.39,                           88,                             "2,440",        "[165, 155, 154] Gray",         0
    // Venus,      0.72,                           224.7,                          "6,052",        "[227, 158, 28] Yellow-White",  0
    // Earth,      1.00,                           365.2,                          "6,371",        "[43, 101, 236] Blue/Green",    0
    // Mars,       1.52,                           687,                            "3,390",        "[193, 68, 14] Red-Orange",     0
    // Jupiter,    5.20,                           "4,331",                        "69,911",       "[216, 202, 157] Brown/Tan",    4
    // Saturn,     9.54,                           "10,747",                       "58,232",       "[191, 171, 119] Pale Gold",    7 (Main Groups)
    // Uranus,     19.22,                          "30,589",                       "25,362",       "[209, 231, 231] Pale Blue",    13
    // Neptune,    30.06,                          "59,800",                       "24,622",       "[63, 115, 255] Bright Blue",   5

    /* This is the design choice detail
     * I decide to keep the radius squared scale linearly to the original data's radius
     * And make further bodies viewable

    Body       | Radius     | Orbit Center    | Gap to Prev
    ------------------------------------------------------------
    Sun        | 48.44      | 0.0000          | N/A
    Mercury    | 9.29       | 78.2239         | 20.5
    Venus      | 12.04      | 120.0475        | 20.5
    Earth      | 12.22      | 164.8008        | 20.5
    Mars       | 10.20      | 207.7171        | 20.5
    Jupiter    | 24.22      | 262.6368        | 20.5
    Saturn     | 22.99      | 330.3424        | 20.5
    Uranus     | 18.13      | 391.9565        | 20.5
    Neptune    | 17.97      | 448.5588        | 20.5

    */

    pub fn new()->Self {
        let mut planets: Vec<CosmicBody> = vec![
            // Mercury
            CosmicBody::new(Vec3::X *  102.4429,    88, Vec3::Z, Vec3::new(165.0, 155.0, 154.0) / 255.0, 9.29 + 5.0),
            // Venus
            CosmicBody::new(Vec3::X *  144.2665,   225, Vec3::Z, Vec3::new(227.0, 158.0,  28.0) / 255.0, 12.04 + 5.0),
            // Earth
            CosmicBody::new(Vec3::X *  189.0198,   365, Vec3::Z, Vec3::new( 43.0, 101.0, 236.0) / 255.0, 12.22 + 5.0),
            // Mars
            CosmicBody::new(Vec3::X * 231.9361,   687, Vec3::Z,  Vec3::new(193.0,  68.0,  14.0) / 255.0, 10.20 + 5.0),
            // Jupiter
            CosmicBody::new(Vec3::X * 286.8558,  4331, Vec3::Z,  Vec3::new(216.0, 202.0, 157.0) / 255.0, 24.22 + 5.0),
            // Saturn
            CosmicBody::new(Vec3::X * 354.5613, 10747, Vec3::Z,  Vec3::new(191.0, 171.0, 119.0) / 255.0, 22.99 + 5.0),
            // Uranus
            CosmicBody::new(Vec3::X * 416.1755, 30589, Vec3::Z,  Vec3::new(209.0, 231.0, 231.0) / 255.0, 18.13 + 5.0),
            // Neptune
            CosmicBody::new(Vec3::X * 472.7778, 59800, Vec3::Z,  Vec3::new( 63.0, 115.0, 255.0) / 255.0, 17.97 + 5.0),
        ];
        
        let mut days_passed: Vec<u32> = vec![0; 8];

        // randomize original position
        let mut rng = rand::rng();
        for i in 0..8 {
            let days_passed_curr: u32 = rng.random_range(0..planets[i].days_per_orbit);
            days_passed[i] = days_passed_curr;
            planets[i].orbit(days_passed_curr);
        }

        let sun: CosmicBody = CosmicBody::new(Vec3::ZERO, 0, Vec3::ZERO, Vec3::new(255.0, 215.0, 0.0) / 255.0, 72.66 + 5.0);

        let mut orbit_triangles: Vec<(Vertex, Vertex, Vertex)> = vec![];
        let orbit_line_counts: f32 = 1000.0;
        let orbit_color: Vec3 = Vec3::new(1.0, 1.0, 1.0);
        let orbit_line_width: f32 = 3.0;


        Self { planets, sun, days_passed, orbit_triangles }
    }

    pub fn orbit(&mut self, day: u32) {
        for i in 0..self.planets.len() {
            self.days_passed[i] += day;
            self.planets[i].orbit(self.days_passed[i]);
        }
    }

}
