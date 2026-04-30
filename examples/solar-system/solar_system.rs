use crate::planet::Planet;
use std::rc::Rc;

pub struct SolarSystem {
    pub planets: Rc<[Planet]>,
    pub sun: Planet,
    pub names: Rc<[Rc<str>]>,
    pub radii: Rc<[f32]>,
    pub orbit_radii: Rc<[f32]>,
    pub rotational_velocity: Rc<[f32]>,
    pub orbital_velocity: Rc<[f32]>,
    t: f32,
}

impl SolarSystem {
    pub fn new() {
        let names: Rc<[Rc<str>]> = Rc::from([
            Rc::from("mercury"),
            Rc::from("venus"),
            Rc::from("earth"),
            Rc::from("mars"),
            Rc::from("jupiter"),
            Rc::from("saturn"),
            Rc::from("uranus"),
            Rc::from("neptune")
        ]);
        // Not to scale
        let radii: Rc<[f32]> = Rc::from([0.5, 0.6, 1.0, 0.8, 2.0, 1.5, 1.3, 1.3]);
        let orbit_radii: Rc<[f32]> = Rc::from([]);

    }
}
