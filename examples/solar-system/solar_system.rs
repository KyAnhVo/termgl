use glam::Vec3;
use termgl::graphics::PointLightSource;

use crate::planet::Planet;
use std::rc::Rc;

pub struct SolarSystem {
    pub planets: Vec<Planet>,
    pub sun: Planet,
    pub sun_light: PointLightSource,
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
        let radii: Rc<[f32]> = Rc::from([0.5, 0.75, 1.0, 1.0, 2.0, 1.5, 1.25, 1.25]);
        let orbit_radii: Rc<[f32]> = Rc::from([5.0, 6.75, 9.0, 11.5, 15.0, 20.5, 24.5, 28.0]);
        let rotational_velocity: Rc<[f32]> = Rc::from([8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]);
        let orbital_velocity: Rc<[f32]> =
            Rc::from([80.0, 70.0, 60.0, 50.0, 40.0, 30.0, 20.0, 10.0]);

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
                Vec3::ONE * 150.0,
                Vec3::ONE,
                Vec3::ONE * 0.1,
                Vec3::ZERO,
                termgl::graphics::LightSourceShadingMode::Lambertian,
            ),
            t_scale,
            t: 0.0,
        }
    }

    pub fn simulate(&mut self, dt: f32) {
        self.t += dt;
        for planet in &mut self.planets {
            planet.move_planet(dt, self.t, self.t_scale);
        }
    }
}
