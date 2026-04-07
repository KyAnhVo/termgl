mod graphics;
mod solar_system;

use crate::graphics::mesh::Mesh;
use crate::graphics::options::{LightSourceShadingMode, ShadingMode};
use crate::graphics::pipeline3d::Pipeline3D;
use crate::graphics::point_light_source::PointLightSource;
use crate::graphics::printer::{Printer, PrinterType};
use crate::graphics::projection::Camera;
use crate::graphics::rasterizer::Rasterizer;
use crate::graphics::vertex::{Material, RasterVertex};
use std::f32::consts::PI;
use std::io::{Write, stdout};

use std::thread;
use std::time::Duration;

use crossterm::terminal;
use glam::{Mat3, Mat4, Vec3, Vec4};

fn main() {
    let (width_u16, height_u16) = terminal::size().unwrap();
    let (width, height) = (width_u16 as usize, height_u16 as usize * 2);
    // test_rast(width, height);
    test_pipeline();
}

fn test_rast(w: usize, h: usize) {
    let v1: RasterVertex =
        RasterVertex::new(Vec4::new(-1.0, 0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
    let v2: RasterVertex =
        RasterVertex::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
    let v3: RasterVertex =
        RasterVertex::new(Vec4::new(0.0, -1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));

    let mut rast: Rasterizer = Rasterizer::new(w, h, Vec3::new(0.0, 0.1, 0.3));

    let mut printer: Printer = Printer::new(PrinterType::Color, w, h);
    printer.print(&rast.frame_buff);
    stdout().write_all(&printer.buff).unwrap();
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
        (CosmicBody::rot_x(f32::consts::PI / 3.0) * (Vec3::Z * -1000.0)).extend(1.0), 
        f32::consts::PI / 4.0, 
        width as f32 / height as f32
    );
    mesh2.create_sphere(20, 20, 3.0, Vec3::new(0.0, 0.0, 1.0));

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

        pipeline.print();
        thread::sleep(Duration::from_millis(10));
    }
}
