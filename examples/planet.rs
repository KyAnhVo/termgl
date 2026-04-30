use glam::{Mat3, Vec3};
use std::f32::consts::PI;
use std::thread::sleep;
use std::time;
use termgl::graphics::{
    Camera, LightSourceShadingMode, Material, Mesh, Pipeline3D, PointLightSource, PrinterType,
    ShadingMode,
};

fn main() {
    // Mesh with texture map
    let material: Material = Material::new(Vec3::ONE * 0.1, 0.01, 500.0);
    let mut mesh1: Mesh = Mesh::create_sphere(0.5, Vec3::X, material, Vec3::ONE, 20, 20);
    mesh1.add_texture_map("examples/assets/earth_bw.jpg");
    let mut mesh2: Mesh = Mesh::create_sphere(0.5, Vec3::NEG_X, material, Vec3::ONE, 20, 20);
    mesh2.add_texture_map("examples/assets/earth_bw.jpg");

    let light: PointLightSource = PointLightSource::new(
        Vec3::NEG_Z * 1.0,
        None,
        Vec3::ONE,
        Vec3::new(0.7, 0.7, 0.7) * 20.0,
        Vec3::ZERO,
        Vec3::ONE,
        LightSourceShadingMode::Lambertian,
    );

    let camera: Camera = Camera::new(
        Vec3::Y.extend(0.0),
        Vec3::Z.extend(0.0),
        (Vec3::NEG_Z * 2.0).extend(1.0),
        PI / 4.0,
    );

    let shading_mode: ShadingMode = ShadingMode::Phong;
    let printer_type: PrinterType = PrinterType::Color;

    let mut pipeline: Pipeline3D = Pipeline3D::new(
        Vec3::new(0.0, 0.0, 0.07),
        printer_type,
        camera,
        shading_mode,
    );

    pipeline.shader.add_point_light_source(light);

    let rotation: Mat3 = Mat3::from_rotation_y(PI / 200.0);

    loop {
        let start = time::Instant::now();

        pipeline.start_frame();
        mesh1.rotate(rotation.clone());
        mesh1.finalize_mesh();

        pipeline.render_mesh(&mut mesh1);
        mesh2.rotate(rotation.clone());
        mesh2.finalize_mesh();
        pipeline.render_mesh(&mut mesh2);

        pipeline.end_frame();

        let elapsed = start.elapsed();
        sleep(
            time::Duration::from_millis(10)
                .checked_sub(elapsed)
                .unwrap_or_default(),
        );
    }
}
