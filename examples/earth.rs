use glam::{Mat3, Vec2, Vec3};
use std::f32::consts::PI;
use std::thread::sleep;
use std::time;
use termgl::{
    Camera, LightSourceShadingMode, Material, Mesh, Pipeline3D, PointLightSource, PrinterType,
    ShadingMode, Vertex, VertexIndices,
};

fn main() {
    let mut mesh: Mesh = create_sphere(0.5, Vec3::Z, 20, 20);
    mesh.add_texture_map("assets/earth.jpg");

    let light: PointLightSource = PointLightSource::new(
        Vec3::new(1.0, 0.0, -0.5) * 0.5,
        None,
        Vec3::ONE,
        Vec3::new(0.0, 0.0, 0.7) * 20.0,
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

fn create_sphere(rad: f32, origin: Vec3, lat: usize, long: usize) -> Mesh {
    let material: Material = Material::new(Vec3::ZERO, Vec3::ONE * 0.01, 0.005);
    let mut mesh: Mesh = Mesh::new(origin, Mat3::IDENTITY, material, true);

    // vertices + normals + uvs
    // lat = rings (pole to pole), long = slices around
    for i in 0..=lat {
        let phi: f32 = PI * i as f32 / lat as f32; // [0, PI]
        for j in 0..=long {
            let theta: f32 = 2.0 * PI * j as f32 / long as f32; // [0, 2PI]

            let x: f32 = rad * phi.sin() * theta.cos();
            let y: f32 = rad * phi.cos();
            let z: f32 = rad * phi.sin() * theta.sin();

            let pos: Vec3 = Vec3::new(x, y, z);
            let normal: Vec3 = pos.normalize(); // outward normal

            let u: f32 = j as f32 / long as f32; // [0, 1] longitude
            let v: f32 = i as f32 / lat as f32; // [0, 1] latitude

            mesh.add_vertex(Vertex::new(pos, Vec3::ONE));
            mesh.add_normal(normal.extend(0.0));
            mesh.add_uv(Vec2::new(u, v));
        }
    }

    // triangles
    // each quad (i, j) -> (i+1, j) -> (i, j+1) -> (i+1, j+1)
    // ring i has (long+1) verts
    for i in 0..lat {
        for j in 0..long {
            let row: usize = long + 1;
            let tl: usize = i * row + j;
            let tr: usize = tl + 1;
            let bl: usize = tl + row;
            let br: usize = bl + 1;

            let mk = |vi: usize| VertexIndices::new(vi, vi, vi);

            // upper triangle of quad
            mesh.add_triangle(mk(tl), mk(bl), mk(tr));
            // lower triangle of quad
            mesh.add_triangle(mk(tr), mk(bl), mk(br));
        }
    }

    mesh
}
