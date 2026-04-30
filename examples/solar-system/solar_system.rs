use glam::Vec3;
use termgl::graphics::PointLightSource;

use crate::planet::Planet;
use std::rc::Rc;

pub struct SolarSystem {
    pub planets: Vec<Planet>,
    pub sun: Planet,
    pub sun_light: PointLightSource,
    pub names: Rc<[Rc<str>]>,
    pub radii: Rc<[f32]>,
    pub orbit_radii: Rc<[f32]>,
    pub rotational_velocity: Rc<[f32]>,
    pub orbital_velocity: Rc<[f32]>,
    t_scale: f32,
    t: f32,
}

impl SolarSystem {
    pub fn new(t_scale: f32) -> Self {
        let names: Rc<[Rc<str>]> = Rc::from([
            Rc::from("mercury"),
            Rc::from("venus"),
            Rc::from("earth"),
            Rc::from("mars"),
            Rc::from("jupiter"),
            Rc::from("saturn"),
            Rc::from("uranus"),
            Rc::from("neptune"),
        ]);
        // Not to scale
        let radii: Rc<[f32]> = Rc::from([0.5, 0.6, 1.0, 0.8, 2.0, 1.5, 1.3, 1.3]);
        let orbit_radii: Rc<[f32]> = Rc::from([6.0, 9.0, 12.0, 15.0, 18.0, 21.0, 24.0, 30.0]);
        let rotational_velocity: Rc<[f32]> = Rc::from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let orbital_velocity: Rc<[f32]> =
            Rc::from([10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]);

        let mut planet_vec: Vec<Planet> = vec![];
        for i in 0..8 {
            planet_vec.push(Planet::new(
                names[i].clone(),
                radii[i],
                orbit_radii[i],
                rotational_velocity[i],
                orbital_velocity[i],
                false,
            ));
        }

        let sun: Planet = Planet::new(Rc::from("sun"), 4.0, 0.0, 0.0, 0.0, true);
        Self {
            planets: planet_vec,
            sun: sun,
            sun_light: PointLightSource::new(
                Vec3::ZERO,
                None,
                Vec3::ONE,
                Vec3::ONE,
                Vec3::ONE * 0.1,
                Vec3::ZERO,
                termgl::graphics::LightSourceShadingMode::Lambertian,
            ),
            names,
            radii,
            orbit_radii,
            rotational_velocity,
            orbital_velocity,
            t_scale,
            t: 0.0,
        }
    }

    pub fn simulate(&mut self, dt: f32) {
        self.t += dt;
        for planet in &mut self.planets {
            planet.move_planet(self.t, self.t_scale);
        }
    }
}
