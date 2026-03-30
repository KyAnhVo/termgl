mod graphics;
mod solar_system;

use crate::graphics::mesh::Mesh;
use crate::graphics::options::{LightSourceShadingMode, ShadingMode};
use crate::graphics::pipeline3d::Pipeline3D;
use crate::graphics::point_light_source::PointLightSource;
use crate::graphics::printer::{Printer, PrinterType};
use crate::graphics::projection::Camera;
use crate::graphics::rasterizer::Rasterizer;
use crate::graphics::vertex::{Material, RasterVertex, Vertex};
use std::f32::consts::PI;
use std::io::{Write, stdout};

use crossterm::terminal;
use glam::{Mat3, Vec3, Vec4};

fn main() {
    let (width_u16, height_u16) = terminal::size().unwrap();
    let (width, height) = (width_u16 as usize, height_u16 as usize * 2);
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
    rast.clear();

    let mut printer: Printer = Printer::new(PrinterType::Color, w, h);
    printer.print(&mut rast.frame_buff);
    stdout().write_all(&printer.buff).unwrap();
}

fn test_pipeline() {
    let mut pipeline: Pipeline3D = Pipeline3D::new(
        Vec3::new(0.0, 0.0, 0.1),
        PrinterType::Color,
        Camera::new(
            Vec3::Y.extend(0.0),
            Vec3::Z.extend(0.0),
            (Vec3::Z * -5.0).extend(1.0),
            PI / 4.0,
        ),
    );
    pipeline
        .shader
        .add_point_light_source(PointLightSource::new(
            Vec3::Z * -5.0,
            None,
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.8, 0.8, 0.8),
            Vec3::new(0.1, 0.1, 0.1),
            Vec3::ZERO,
            LightSourceShadingMode::Lambertian,
        ));

    let mut mesh: Mesh = Mesh::new(
        Vec3::Z * 3.0,
        Mat3::IDENTITY,
        Material {
            ks: Vec3 {
                x: 1.0,
                y: 0.86,
                z: 0.57,
            },
            ka: Vec3 {
                x: 0.25,
                y: 0.19,
                z: 0.08,
            },
            p: 50.0,
        },
        true,
    );

    mesh.create_sphere(1.0, Vec3::new(1.0, 0.76, 0.33));
    mesh.finalize_normals();

    loop {
        pipeline.resize();
        pipeline.rasterizer.clear();
        pipeline.render_mesh(&mut mesh, ShadingMode::Phong);
        pipeline.print();
    }
}
