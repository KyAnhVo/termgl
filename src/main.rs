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

fn test_pipeline() {
    let mut pipeline: Pipeline3D = Pipeline3D::new(
        Vec3::new(0.0, 0.0, 0.1),
        PrinterType::Color,
        Camera::new(
            Vec3::X.extend(0.0),
            Vec3::Y.extend(0.0) * 1.0,
            (Vec3::Y * -30.0).extend(1.0),
            PI / 4.0,
        ),
        ShadingMode::Phong,
    );

    let intensity: f32 = 5.0;
    pipeline
        .shader
        .add_point_light_source(PointLightSource::new(
            Vec3::ZERO,
            None,
            Vec3::ONE * intensity * 10.0,
            Vec3::ONE * 0.2 * intensity,
            Vec3::ONE * 0.1 * intensity,
            Vec3::ZERO,
            LightSourceShadingMode::Lambertian,
        ));

    let mut mesh: Mesh = Mesh::new(
        Vec3::Z * 10.0,
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
        false,
    );
    mesh.create_sphere(20, 20, 2.0, Vec3::new(1.0, 0.76, 0.33));

    let mut mesh2: Mesh = Mesh::new(
        Vec3::new(5.0, 0.0, 10.0),
        Mat3::IDENTITY,
        Material::new(
            Vec3::new(1.0, 0.86, 0.57),
            Vec3::new(0.25, 0.19, 0.08),
            50.0,
        ),
        false,
    );
    mesh2.create_sphere(20, 20, 3.0, Vec3::new(0.0, 0.0, 1.0));

    loop {
        mesh.move_origin(Mat4::from_mat3(Mat3::from_rotation_y(PI / 100.0)));
        pipeline.resize();
        pipeline.rasterizer.clear();
        pipeline.render_mesh(&mut mesh);
        pipeline.render_mesh(&mut mesh2);

        pipeline.print();
        thread::sleep(Duration::from_millis(10));
    }
}
