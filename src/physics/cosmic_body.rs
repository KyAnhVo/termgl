use std::f32;
use rand::{RngExt};
use glam::{Mat3, Vec3};

use crate::graphics::triangle::Color;

#[derive(Clone, Copy)]
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
    pub color: Color,

    /// radius to draw the cosmic body
    pub radius: f32,
}

impl CosmicBody {

    pub fn new(
        original_pos: Vec3, 
        days_per_orbit: u32,
        rotational_normal: Vec3, 
        color: Color, 
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
            orbital_angular_velocity,
            days_per_orbit,
            rotational_normal: rotational_normal.normalize(),
            color,
            radius,
        }
    }

    pub fn orbit(&mut self, days_passed: u32) {
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
}

pub struct CosmicSimulator {
    pub planets: Vec<CosmicBody>,
    pub sun: CosmicBody,
    pub days_passed: Vec<u32>,
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
    Body       | Radius     | Orbit Center    | Gap to Prev
    ------------------------------------------------------------
    Sun        | 31.48      | 0.0000          | N/A
    Mercury    | 9.29       | 41.2706         | 0.5
    Venus      | 12.04      | 63.0942         | 0.5
    Earth      | 12.22      | 87.8475         | 0.5
    Mars       | 10.20      | 110.7639        | 0.5
    Jupiter    | 24.22      | 145.6835        | 0.5
    Saturn     | 22.99      | 193.3891        | 0.5
    Uranus     | 18.13      | 235.0032        | 0.5
    Neptune    | 17.97      | 271.6055        | 0.5
    */

    pub fn new()->Self {
        let mut planets: Vec<CosmicBody> = vec![
            // Mercury
            CosmicBody::new(Vec3::X *  41.2706,    88, Vec3::Z, Color::new(165, 155, 154), 9.29),
            // Venus
            CosmicBody::new(Vec3::X *  63.0942,   225, Vec3::Z, Color::new(227, 158,  28), 12.04),
            // Earth
            CosmicBody::new(Vec3::X *  87.8475,   365, Vec3::Z, Color::new( 43, 101, 236), 12.22),
            // Mars
            CosmicBody::new(Vec3::X * 110.7639,   687, Vec3::Z, Color::new(193,  68,  14), 10.20),
            // Jupiter
            CosmicBody::new(Vec3::X * 145.6835,  4331, Vec3::Z, Color::new(216, 202, 157), 24.22),
            // Saturn
            CosmicBody::new(Vec3::X * 193.3891, 10747, Vec3::Z, Color::new(191, 171, 119), 22.99),
            // Uranus
            CosmicBody::new(Vec3::X * 235.0032, 30589, Vec3::Z, Color::new(209, 231, 231), 18.13),
            // Neptune
            CosmicBody::new(Vec3::X * 271.6055, 59800, Vec3::Z, Color::new( 63, 115, 255), 17.97),
        ];
        
        let mut days_passed: Vec<u32> = vec![0; 8];

        // randomize original position
        let mut rng = rand::rng();
        for i in 0..8 {
            let days_passed_curr: u32 = rng.random_range(0..planets[i].days_per_orbit);
            days_passed[i] = days_passed_curr;
            planets[i].orbit(days_passed_curr);
        }

        let sun: CosmicBody = CosmicBody::new(Vec3::ZERO, 0, Vec3::ZERO, Color::new(255, 215, 0), 31.48);

        Self { planets, sun, days_passed }
    }
}
