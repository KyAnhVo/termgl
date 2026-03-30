mod graphics;
mod physics;

use crate::graphics::projection::Camera;
use crate::graphics::triangle::{Triangle, RasterTriangle};
use crate::graphics::printer::{Printer, PrinterType};
use crate::graphics::rasterizer::Rasterizer;

use crate::physics::cosmic_body::{CosmicBody, CosmicSimulator};

use std::env::args;
use std::f32;
use std::{thread, time};
use std::io::{stdout, Write};
use glam::{Mat4, Vec3};
use crossterm::terminal;

fn main() {
    let (width_u16, height_u16) = terminal::size().unwrap();
    let (width, height) = (width_u16 as usize, height_u16 as usize * 2);

    test_planets(width, height, 2);
}

fn test_planets(width: usize, height: usize, day_count_incremental: u32) {
    let print_type: PrinterType = if args().collect::<Vec<String>>()[1] == "-a" {
        PrinterType::Ascii
    } else {
        PrinterType::Color
    };
    let mut rasterizer: Rasterizer = Rasterizer::new(width, height);
    let mut cosmic_simulator = CosmicSimulator::new();

    let cam: Camera = Camera::new(
        (CosmicBody::rot_x(f32::consts::PI / 3.0) * Vec3::Y).extend(0.0), 
        (CosmicBody::rot_x(f32::consts::PI / 3.0) * Vec3::Z).extend(0.0), 
        (CosmicBody::rot_x(f32::consts::PI / 3.0) * (Vec3::Z * -1500.0)).extend(1.0), 
        f32::consts::PI / 4.0, 
        width as f32 / height as f32
    );
    let projection: Mat4 = cam.m_perspective(0.01, 500.0) * cam.m_view();

    loop {
        let (width_u16, height_u16) = terminal::size().unwrap();
        let (width, height) = (width_u16 as usize, height_u16 as usize * 2);

        rasterizer.resize(width, height);

        cosmic_simulator.orbit(day_count_incremental);
        let triangles: Vec<Triangle> = cosmic_simulator.calculate_triangles(cam);
        let mut raster_triangles: Vec<RasterTriangle> = vec![];
        for triangle in triangles.iter() {
            raster_triangles.push(RasterTriangle::from_world_view(*triangle, projection));
        }
        for raster_triangle in raster_triangles.iter() {
            rasterizer.render_triangle(raster_triangle);
        }
        let mut color_printer = Printer::new(print_type, width, height);
        color_printer.print(&mut rasterizer.frame_buff);
        stdout().write_all(&color_printer.buff).unwrap();
        rasterizer.clear();
        thread::sleep(time::Duration::from_millis(10));
    }

}
