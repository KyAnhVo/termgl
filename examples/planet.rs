use glam::{Mat3, Vec2, Vec3};
use std::env;
use std::f32::consts::PI;
use std::thread::sleep;
use std::time;
use termgl::graphics::{
    Camera, LightSourceShadingMode, Material, Mesh, Pipeline3D, PointLightSource, PrinterType,
    ShadingMode,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: planet <planet>");
        return;
    }
    let planet = &args[1].to_ascii_lowercase();
    let planet_trimmed = planet.trim();

    // Mesh with texture map
    let material: Material = Material::new(Vec3::ONE * 0.1, Vec3::ONE * 0.01, 10000.0);
    let mut mesh: Mesh = Mesh::create_sphere(0.5, Vec3::Z, material, Vec3::ONE, 20, 20);
    mesh.add_texture_map(&format!("examples/assets/{}.jpg", planet_trimmed));

    let light: PointLightSource = PointLightSource::new(
        Vec3::new(1.0, 0.0, -0.5) * 0.5,
        None,
        Vec3::ONE,
        Vec3::new(0.7, 0.7, 0.7) * 20.0,
        Vec3::ZERO,
        Vec3::ONE,
        LightSourceShadingMode::Lambertian,
    );

    let camera: Camera = Camera::new(
        Vec3::Y.extend(0.0),
        Vec3::NEG_Z.extend(0.0),
        Vec3::NEG_Z.extend(1.0),
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
        mesh.rotate(rotation.clone());
        mesh.finalize_mesh();
        pipeline.resize();
        pipeline.render_mesh(&mut mesh);
        pipeline.print();
        let elapsed = start.elapsed();
        sleep(
            time::Duration::from_millis(10)
                .checked_sub(elapsed)
                .unwrap_or_default(),
        );
    }
}
