mod planet;
mod solar_system;

use std::{f32::consts::PI, thread::sleep, time};

use glam::Vec3;
use solar_system::SolarSystem;
use termgl::graphics::{Camera, Pipeline3D, PrinterType};

fn main() {
    let t_scale: f32 = 1.0;
    let mut solar_system: SolarSystem = SolarSystem::new(t_scale);
    let printer_type: PrinterType = PrinterType::Color;

    let cam_pos = Vec3::new(1.0, 0.5, 1.0) * 45.0;
    let camera: Camera = Camera::new(
        Vec3::Y.extend(0.0),
        (-cam_pos).normalize().extend(0.0),
        cam_pos.extend(1.0),
        PI / 4.0,
    );
    let mut pipeline: Pipeline3D = Pipeline3D::new(
        Vec3::ZERO,
        printer_type,
        camera,
        termgl::graphics::ShadingMode::Phong,
    );
    pipeline
        .shader
        .add_point_light_source(solar_system.sun_light.clone());
    loop {
        pipeline.start_frame();
        solar_system.simulate(1.0 / 1000.0);
        pipeline.render_mesh(&mut solar_system.sun.mesh);
        for planet in &mut solar_system.planets {
            pipeline.render_mesh(&mut planet.mesh);
            pipeline.render_mesh(&mut planet.orbit_line);
            match &mut planet.ring_mesh {
                Some(ring) => pipeline.render_mesh(ring),
                None => {}
            }
        }
        pipeline.end_frame();

        sleep(time::Duration::from_millis(10));
    }
}
