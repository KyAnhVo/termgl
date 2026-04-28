use crate::graphics::{
    projection::Camera,
    uv_map::{HeightMap, NormalMap, UVMap},
    vertex::{Material, RasterVertex, Vertex},
};
use glam::{Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use std::f32::consts::PI;

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

    /// finalize normals essentially transforms the normal to world space.
    /// So if  no movement/spinning, it still is in that same position,
    /// thus we use this var to indicate if it has changed.
    no_change: bool,
}

impl Mesh {
    pub fn new(origin: Vec3, orthonormal_basis: Mat3, material: Material, no_shade: bool) -> Self {
        Self {
            origin,
            orthonormal_basis,
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
            no_change: false,
        }
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
        let mut mesh: Mesh = Mesh::new(origin, Mat3::IDENTITY, material, false);

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

                mesh.add_vertex(Vertex::new(pos, color));
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
}
