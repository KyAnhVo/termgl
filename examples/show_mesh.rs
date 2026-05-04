use std::{env, f32, io};

use glam::{Mat3, Vec3};
use termgl::graphics::{Camera, Mesh, Pipeline3D, PrinterType, ShadingMode};

fn main() -> io::Result<()> {
    let mut meshes: Vec<Mesh> = Mesh::import_obj("examples/assets/suitcase.obj", None)?;
    println!("Import mesh don");

    let cam_pos: Vec3 = Vec3::X * 10.0;
    let camera: Camera = Camera::new(
        Vec3::Y.extend(0.0),
        -cam_pos.normalize().extend(0.0),
        cam_pos.extend(1.0),
        f32::consts::PI / 4.0,
    );

    let shading_mode: ShadingMode = ShadingMode::Phong;
    let printer_type: PrinterType = PrinterType::Ascii;

    let mut pipeline: Pipeline3D = Pipeline3D::new(
        Vec3::new(0.0, 0.0, 0.07),
        printer_type,
        camera,
        shading_mode,
    );

    let rotation: Mat3 = Mat3::from_rotation_y(f32::consts::PI / 200.0);
    pipeline.start_frame();
    pipeline.render_mesh(&mut meshes[0]);
    pipeline.end_frame();

    Ok(())
}
