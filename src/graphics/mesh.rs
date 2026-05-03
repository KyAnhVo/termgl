use crate::graphics::{
    camera::Camera,
    uv_map::{HeightMap, NormalMap, UVMap},
    vertex::{Material, RasterVertex, Vertex},
};
use glam::{Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use std::io;
use std::{
    f32::consts::PI,
    fs::File,
    io::{BufWriter, Write},
};

/// a 3-tuple of indices of vertex, normal, and uv
#[derive(Clone, Copy)]
pub struct VertexIndices {
    pub vertex_ind: usize,
    pub normal_ind: usize,
    pub uv_ind: usize,
}

impl VertexIndices {
    pub fn new(vertex_ind: usize, normal_ind: usize, uv_ind: usize) -> Self {
        Self {
            vertex_ind,
            normal_ind,
            uv_ind,
        }
    }
}

/// An object to be rendered, represented by an EBO and a VAO
#[derive(Clone)]
pub struct Mesh {
    /// The origin of the mesh in object space
    origin: Vec3,

    /// The object space's orthonormal basis
    orthonormal_basis: Mat3,

    /// The vertices in the VAO is in object space
    pub vertices: Vec<Vertex>,

    /// The vertices in the VAO but in world space
    pub vertices_world_space: Vec<Vertex>,

    /// Used for RasterVertex after projection
    pub raster_vertices: Vec<RasterVertex>,

    /// orthogonal of the vertices in object space.
    pub normals: Vec<Vec4>,

    /// vertex orthogonal of vertices in world space.
    pub normals_world_space: Vec<Vec4>,

    /// uv coordinates
    pub uv: Vec<Vec2>,

    /// Each element in the EBO maps to a vertex in
    /// the VAO, and each 3 elements 3x, 3x+1, 3x+2 in
    /// the EBO creates a triangle.
    /// Note: ebo.size() % 3 == 0 must remain true.
    pub triangles: Vec<VertexIndices>,

    /// As of this version, expect the mesh to remain the same material
    pub material: Material,

    ///texture uv map
    pub texture_map: Option<UVMap>,

    /// height uv map
    pub height_map: Option<HeightMap>,

    /// normal uv map
    pub normal_map: Option<NormalMap>,

    /// Some objects might not want to be shaded (e.g. background image,
    /// 2D game with no shading, or a light source)
    pub no_shade: bool,

    /// default color if no uv mapping
    pub default_color: Vec3,

    /// finalize normals essentially transforms the normal to world space.
    /// So if  no movement/spinning, it still is in that same position,
    /// thus we use this var to indicate if it has changed.
    no_change: bool,
}

impl Mesh {
    pub fn new(material: Material, no_shade: bool) -> Self {
        Self {
            origin: Vec3::ZERO,
            orthonormal_basis: Mat3::IDENTITY,
            vertices: vec![],
            vertices_world_space: vec![],
            raster_vertices: vec![],
            normals: vec![],
            normals_world_space: vec![],
            uv: vec![],
            triangles: vec![],
            material,
            texture_map: None,
            height_map: None,
            normal_map: None,
            no_shade,
            default_color: Vec3::new(144.0, 144.0, 144.0) / 255.0,
            no_change: false,
        }
    }

    pub fn get_origin(&self) -> Vec3 {
        self.origin
    }

    pub fn get_orthonormal_basis(&self) -> Mat3 {
        self.orthonormal_basis
    }

    /// Adds a vertex to the mesh.
    pub fn add_vertex(&mut self, v: Vertex) {
        self.vertices.push(v);
        self.no_change = false;
    }

    pub fn add_normal(&mut self, normal: Vec4) {
        self.normals.push(normal);
        self.no_change = false;
    }

    pub fn add_uv(&mut self, uv: Vec2) {
        self.uv.push(uv);
    }

    /// Adds a triangle to the mesh by pushing its vertex indices to the EBO.
    ///
    pub fn add_triangle(&mut self, a: VertexIndices, b: VertexIndices, c: VertexIndices) {
        let vertices_len: usize = self.vertices.len();
        let uv_len: usize = self.uv.len();
        let normals_len: usize = self.normals.len();
        assert!(
            a.vertex_ind < vertices_len
                && b.vertex_ind < vertices_len
                && c.vertex_ind < vertices_len,
        );
        assert!(a.uv_ind < uv_len && b.uv_ind < uv_len && c.uv_ind < uv_len,);
        assert!(
            a.normal_ind < normals_len && b.normal_ind < normals_len && c.normal_ind < normals_len,
        );

        // pretty much assume the compiler will optimize this
        // with even the lowest level of optimization
        self.triangles.append(&mut vec![a, b, c]);
    }

    /// Finalize the mesh before rendering. Must call.
    pub fn finalize_mesh(&mut self) {
        if self.no_change {
            return;
        }

        self.normals_world_space.clear();
        let m_to_world_space: Mat4 = self.m_to_world_space();
        for i in 0..self.normals.len() {
            // orthogonal
            self.normals_world_space
                .push((m_to_world_space * self.normals[i]).normalize());
        }

        let m_to_world_space: Mat4 = self.m_to_world_space();
        self.vertices_world_space.clear();
        for i in 0..self.vertices.len() {
            let mut vertex: Vertex = self.vertices[i].clone();
            vertex.pos = m_to_world_space * vertex.pos;
            self.vertices_world_space.push(vertex);
        }

        self.no_change = true;
    }

    /// Returns the correct uv that is based on the height map, if one exists.
    /// Otherwise, returns the original uv.
    pub fn get_parallax_uv(&self, uv: Vec2, cam: &Camera) -> Vec2 {
        match self.height_map {
            Some(ref height_map) => {
                let view_dir: Vec3 = cam.pos.xyz() / cam.pos.w - self.origin;
                height_map.interpolate_parallax_uv(uv, view_dir)
            }
            None => uv,
        }
    }

    /// adds texture map, panics if path is invalid
    pub fn add_texture_map(&mut self, path: &str) {
        self.texture_map = Some(UVMap::new(path));
    }

    /// adds normal map, panics if path is invalid
    pub fn add_normal_map(&mut self, path: &str) {
        self.normal_map = Some(NormalMap::new(path));
    }

    /// adds height map, panics if path is invalid
    pub fn add_height_map(&mut self, path: &str, height_scale: f32) {
        self.height_map = Some(HeightMap::new(path, height_scale));
    }

    /// return the matrix to transform vertex to world space
    pub fn m_to_world_space(&self) -> Mat4 {
        Mat4::from_cols(
            self.orthonormal_basis.x_axis.extend(0.0),
            self.orthonormal_basis.y_axis.extend(0.0),
            self.orthonormal_basis.z_axis.extend(0.0),
            self.origin.extend(1.0),
        )
    }

    /// rotates the object/mesh's orthonormal basis
    pub fn rotate(&mut self, m_rotate: Mat3) {
        let det: f32 = m_rotate.determinant();
        assert!(
            (1.0 - f32::EPSILON..=1.0 + f32::EPSILON).contains(&det),
            "invalid rotational matrix"
        );
        self.orthonormal_basis = m_rotate * self.orthonormal_basis;
        self.no_change = false;
    }

    /// moves the object/mesh's origin by a specific vector (origin = origin + movement)
    pub fn translate(&mut self, movement: Vec3) {
        if movement == Vec3::ZERO {
            return;
        }
        self.origin += movement;
        self.no_change = false;
    }

    /// multiply the origin by the 4x4 matrix, 3x3 linear transformation
    /// and 3x1 translation.
    /// This is potentially not as good as move_origin_to for some specific
    /// methods that need high accuracy due to f32's small representation space.
    pub fn move_origin(&mut self, rotation: Mat4) {
        self.origin = (rotation * self.origin.extend(1.0)).xyz();
        self.no_change = false;
    }

    /// moves the object/mesh's origin to a specific position in world space.
    pub fn move_origin_to(&mut self, to: Vec3) {
        self.origin = to;
        self.no_change = false;
    }
}

impl Mesh {
    pub fn create_sphere(
        rad: f32,
        origin: Vec3,
        material: Material,
        color: Vec3,
        lat: usize,
        long: usize,
    ) -> Mesh {
        let mut mesh: Mesh = Mesh::new(material, false);
        mesh.default_color = color;
        mesh.move_origin_to(origin);

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

                mesh.add_vertex(Vertex::from_vec3(pos));
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

    pub fn create_ring(
        r_in: f32,
        r_out: f32,
        origin: Vec3,
        material: Material,
        color: Vec3,
        d_theta: f32,
    ) -> Self {
        assert!(r_in > 0.0 && r_out > r_in, "invalid ring radii");

        let mut mesh: Mesh = Mesh::new(material, false);
        mesh.default_color = color;
        mesh.move_origin_to(origin);

        let n: usize = (2.0 * PI / d_theta).round().max(3.0) as usize;

        // Per angular step we emit 4 vertices: top-inner, top-outer,
        // bottom-inner, bottom-outer. Top vertices carry +Y normals,
        // bottom carry -Y, so each face shades correctly regardless of
        // which side the camera is on.
        let n_up: Vec4 = Vec3::Y.extend(0.0);
        let n_down: Vec4 = Vec3::NEG_Y.extend(0.0);

        for i in 0..=n {
            let theta: f32 = 2.0 * PI * i as f32 / n as f32;
            let (s, c): (f32, f32) = (theta.sin(), theta.cos());
            let v: f32 = i as f32 / n as f32;
            let p_in: Vec3 = Vec3::new(r_in * c, 0.0, r_in * s);
            let p_out: Vec3 = Vec3::new(r_out * c, 0.0, r_out * s);

            // top inner
            mesh.add_vertex(Vertex::from_vec3(p_in));
            mesh.add_normal(n_up);
            mesh.add_uv(Vec2::new(0.0, v));
            // top outer
            mesh.add_vertex(Vertex::from_vec3(p_out));
            mesh.add_normal(n_up);
            mesh.add_uv(Vec2::new(1.0, v));
            // bottom inner
            mesh.add_vertex(Vertex::from_vec3(p_in));
            mesh.add_normal(n_down);
            mesh.add_uv(Vec2::new(0.0, v));
            // bottom outer
            mesh.add_vertex(Vertex::from_vec3(p_out));
            mesh.add_normal(n_down);
            mesh.add_uv(Vec2::new(1.0, v));
        }

        let mk = |vi: usize| VertexIndices::new(vi, vi, vi);
        for i in 0..n {
            // 4 verts per step, ordered [t_in, t_out, b_in, b_out]
            let a: usize = 4 * i;
            let b: usize = 4 * (i + 1);
            let (t_in_a, t_out_a, b_in_a, b_out_a) = (a, a + 1, a + 2, a + 3);
            let (t_in_b, t_out_b, b_in_b, b_out_b) = (b, b + 1, b + 2, b + 3);

            // top face (+Y normals)
            mesh.add_triangle(mk(t_in_a), mk(t_out_a), mk(t_in_b));
            mesh.add_triangle(mk(t_out_a), mk(t_out_b), mk(t_in_b));

            // bottom face (-Y normals, reversed winding so it's
            // front-facing from below)
            mesh.add_triangle(mk(b_in_a), mk(b_in_b), mk(b_out_a));
            mesh.add_triangle(mk(b_out_a), mk(b_in_b), mk(b_out_b));
        }

        mesh
    }
}

impl Mesh {
    pub fn import_obj(path: &str) -> Self {
        let mut mesh: Self = Self::new(Material::new(Vec3::ZERO, 0.1, 5.0), false);

        // TODO: implement mesh import

        mesh
    }

    pub fn export_obj(&self, mesh_path: &str, mtl_path: &str) -> io::Result<()> {
        let mesh_file: File = File::create(mesh_path)?;
        let mut mesh_writer: BufWriter<File> = BufWriter::new(mesh_file);

        let material_file: Option<File> = if mtl_path.len() == 0 {
            None
        } else {
            Some(File::create(mtl_path)?)
        };
        let mut material_writer: Option<BufWriter<File>> = match material_file {
            Some(file) => Some(BufWriter::new(file)),
            None => None,
        };
        match &mut material_writer {
            Some(writer) => {
                let material: Material = self.material;
                let kd: Vec3 = self.default_color;
                let ks: Vec3 = material.specular_constant;
                let ka: Vec3 = kd * material.ambient_constant;
                writeln!(writer, "newmtl material")?;
                writeln!(writer, "Ka {} {} {}", ka.x, ka.y, ka.z)?;
                writeln!(writer, "Ks {} {} {}", ks.x, ks.y, ks.z)?;
                writeln!(writer, "Kd {} {} {}", kd.x, kd.y, kd.z)?;

                writeln!(mesh_writer, "mtllib {}", mtl_path)?;
            }
            None => {}
        };

        writeln!(mesh_writer, "\n# Vertices\n")?;
        for vertex in self.vertices.iter() {
            writeln!(
                mesh_writer,
                "v {} {} {}",
                vertex.pos.x / vertex.pos.w,
                vertex.pos.y / vertex.pos.w,
                vertex.pos.z / vertex.pos.w
            )?;
        }

        writeln!(mesh_writer, "\n# uv indices\n")?;
        for uv in self.uv.iter() {
            writeln!(mesh_writer, "vt {} {}", uv.x, uv.y)?;
        }

        writeln!(mesh_writer, "\n# normals\n")?;
        for normal in self.normals.iter() {
            writeln!(mesh_writer, "vn {} {} {}", normal.x, normal.y, normal.z)?;
        }

        writeln!(mesh_writer, "\n# triangles\n")?;
        match &material_writer {
            Some(_) => writeln!(mesh_writer, "usemtl material")?,
            None => {}
        };
        for triangle_indices in self.triangles.chunks_exact(3) {
            let (a_ind, b_ind, c_ind): (VertexIndices, VertexIndices, VertexIndices) = (
                triangle_indices[0],
                triangle_indices[1],
                triangle_indices[2],
            );
            writeln!(
                mesh_writer,
                "f {}/{}/{} {}/{}/{} {}/{}/{}",
                a_ind.vertex_ind + 1,
                a_ind.uv_ind + 1,
                a_ind.normal_ind + 1,
                b_ind.vertex_ind + 1,
                b_ind.uv_ind + 1,
                b_ind.normal_ind + 1,
                c_ind.vertex_ind + 1,
                c_ind.uv_ind + 1,
                c_ind.normal_ind + 1,
            )?;
        }

        Ok(())
    }
}
