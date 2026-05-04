use std::{env, f32, io};

use glam::{Mat3, Vec3};
use std::thread::sleep;
use std::time;
use termgl::{
    graphics::{Camera, Mesh, Pipeline3D, PointLightSource, PrinterType, ShadingMode},
    simplifier::vertex_cluster,
};

fn main() -> io::Result<()> {
    let light_source_0: PointLightSource = PointLightSource::new(
        Vec3::X * 20.0,
        None,
        Vec3::ONE * 50.0,
        Vec3::ONE * 10.0,
        Vec3::ONE,
        Vec3::ONE,
        termgl::graphics::LightSourceShadingMode::Lambertian,
    );

    let light_source_1: PointLightSource = PointLightSource::new(
        Vec3::NEG_X * 20.0,
        None,
        Vec3::ONE * 50.0,
        Vec3::ONE * 10.0,
        Vec3::ONE,
        Vec3::ONE,
        termgl::graphics::LightSourceShadingMode::Lambertian,
    );

    let light_source_2: PointLightSource = PointLightSource::new(
        Vec3::Z * 20.0,
        None,
        Vec3::ONE * 50.0,
        Vec3::ONE * 10.0,
        Vec3::ONE,
        Vec3::ONE,
        termgl::graphics::LightSourceShadingMode::Lambertian,
    );

    let light_source_3: PointLightSource = PointLightSource::new(
        Vec3::NEG_Z * 20.0,
        None,
        Vec3::ONE * 50.0,
        Vec3::ONE * 10.0,
        Vec3::ONE,
        Vec3::ONE,
        termgl::graphics::LightSourceShadingMode::Lambertian,
    );

    let args: Vec<String> = std::env::args().collect();
    let file: &str = args[1].as_str();
    let mut meshes: Vec<Mesh> =
        Mesh::import_obj(format!("examples/assets/{}.obj", file).as_str(), None)?;
    meshes[0].no_shade = false;
    let mut mesh: Mesh = meshes.remove(0);
    mesh.move_origin_to(Vec3::X * 5.0);
    mesh.scale_to(50.0, 20.0, 20.0);
    let mut simplifed_mesh: Mesh = vertex_cluster(&mesh, 0.2);
    simplifed_mesh.export_obj(
        format!("examples/assets/simplified_{}.obj", file).as_str(),
        "",
    )?;
    simplifed_mesh.material.diffuse_constant = Vec3::ONE;
    simplifed_mesh.move_origin_to(Vec3::NEG_X * 5.0);

    let mut cam_pos: Vec3 = Vec3::new(1.0, 1.0, 1.0) * 20.0;
    let camera: Camera = Camera::new(
        Vec3::Y.extend(0.0),
        -cam_pos.normalize().extend(0.0),
        cam_pos.extend(1.0),
        f32::consts::PI / 4.0,
    );

    let shading_mode: ShadingMode = ShadingMode::Phong;
    let printer_type: PrinterType = PrinterType::Color;

    let mut pipeline: Pipeline3D = Pipeline3D::new(
        Vec3::new(0.0, 0.0, 0.07),
        printer_type,
        camera,
        shading_mode,
    );

    pipeline.shader.add_point_light_source(light_source_0);
    pipeline.shader.add_point_light_source(light_source_1);
    pipeline.shader.add_point_light_source(light_source_2);
    pipeline.shader.add_point_light_source(light_source_3);

    let rotation: Mat3 = Mat3::from_rotation_y(f32::consts::PI / 200.0);
    loop {
        let start = time::Instant::now();

        cam_pos = rotation * cam_pos;
        pipeline.camera.look_at(Vec3::ZERO, cam_pos, Vec3::Y);
        pipeline.start_frame();
        pipeline.render_mesh(&mut mesh);
        pipeline.render_mesh(&mut simplifed_mesh);
        pipeline.end_frame();

        let elapsed = start.elapsed();
        sleep(
            time::Duration::from_millis(10)
                .checked_sub(elapsed)
                .unwrap_or_default(),
        );
    }
}
