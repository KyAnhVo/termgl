use crate::graphics::vertex::{Material, RasterVertex, Vertex};
use glam::{Mat3, Mat4, Vec3, Vec4, Vec4Swizzles};

/// An object to be rendered, represented by an EBO and a VAO
#[derive(Clone)]
pub struct Mesh {
    /// The origin of the mesh in object space
    origin: Vec3,

    /// The object space's orthonormal basis
    orthonormal_basis: Mat3,

    /// The vertices in the VAO is in object space
    pub vao: Vec<Vertex>,

    /// The vertices in the VAO but in world space
    pub vao_world_space: Vec<Vertex>,

    /// Each element in the EBO maps to a vertex in
    /// the VAO, and each 3 elements 3x, 3x+1, 3x+2 in
    /// the EBO creates a triangle.
    /// Note: ebo.size() % 3 == 0 must remain true.
    pub ebo: Vec<usize>,

    /// Used for RasterVertex after projection
    pub projected_vao: Vec<RasterVertex>,

    /// orthogornals of the vertices in object space.
    pub vertex_orthogonals: Vec<Vec4>,

    /// vertex orthogornals of vertices in world space.
    pub v_orthogonals_world_space: Vec<Vec4>,

    /// As of this version, expect the mesh to remain the same material
    pub material: Material,

    /// Some objects might not want to be shaded (e.g. background image,
    /// 2D game with no shading, or a light source)
    pub no_shade: bool,

    /// finalize normals essentially transforms the normal to world space.
    /// So if  no movement/spinning, it still is in that same position.
    /// Thus we use this var to indicate if it has changed.
    no_change: bool,
}

impl Mesh {
    pub fn new(origin: Vec3, orthonormal_basis: Mat3, material: Material, no_shade: bool) -> Self {
        Self {
            origin,
            orthonormal_basis,
            vao: vec![],
            vao_world_space: vec![],
            ebo: vec![],
            projected_vao: vec![],
            vertex_orthogonals: vec![],
            v_orthogonals_world_space: vec![],
            material,
            no_shade,
            no_change: false,
        }
    }

    /// Adds a vertex to the mesh.
    pub fn add_vertex(&mut self, v: Vertex) {
        self.vao.push(v);
        self.vertex_orthogonals.push(Vec4::ZERO);
    }

    /// Adds a triangle to the mesh by pushing its vertex indices to the EBO.
    ///
    pub fn add_triangle(&mut self, a: usize, b: usize, c: usize) {
        let v_count: usize = self.vao.len();
        assert!(
            a < v_count && b < v_count && c < v_count,
            "triangle index out of bound"
        );
        self.ebo.push(a);
        self.ebo.push(b);
        self.ebo.push(c);

        let va: Vec4 = self.vao[a].pos;
        let vb: Vec4 = self.vao[b].pos;
        let vc: Vec4 = self.vao[c].pos;

        let ab: Vec4 = vb - va;
        let ac: Vec4 = vc - va;
        let n: Vec4 = ab.xyz().cross(ac.xyz()).extend(0.0);

        self.vertex_orthogonals[a] += n;
        self.vertex_orthogonals[b] += n;
        self.vertex_orthogonals[c] += n;
    }

    /// Normalizes the vertex orthogonals after all triangles have been added.
    pub fn finalize_normals(&mut self) {
        if self.no_change {
            return;
        }

        self.v_orthogonals_world_space.clear();
        self.vao_world_space.clear();
        let m_to_world_space: Mat4 = self.m_to_world_space();
        for i in 0..self.vertex_orthogonals.len() {
            // orthogornal
            self.v_orthogonals_world_space
                .push((m_to_world_space * self.vertex_orthogonals[i]).normalize());

            // vertex
            let mut v_world_space: Vertex = self.vao[i];
            v_world_space.pos = m_to_world_space * v_world_space.pos;
            self.vao_world_space.push(v_world_space);
        }
        self.no_change = true;
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

    /// moves the object/mesh's origin
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

    pub fn move_origin_to(&mut self, to: Vec3) {
        self.origin = to;
        self.no_change = false;
    }

    // Utility functions

    /// Make mesh a sphere with radius centered around the mesh's origin
    pub fn create_sphere(
        &mut self,
        longtitudes: usize,
        latitudes: usize,
        radius: f32,
        color: Vec3,
    ) {
        self.vao.clear();
        self.ebo.clear();
        self.vertex_orthogonals.clear();

        let sector_step = 2.0 * std::f32::consts::PI / longtitudes as f32;
        let stack_step = std::f32::consts::PI / latitudes as f32;

        for i in 0..=latitudes {
            let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
            let xy = radius * stack_angle.cos();
            let z = radius * stack_angle.sin();

            for j in 0..=longtitudes {
                let sector_angle = j as f32 * sector_step;

                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                let pos = Vec3::new(x, y, z);

                self.vao.push(Vertex::new(pos, color));

                self.vertex_orthogonals.push(pos.extend(0.0).normalize());
            }
        }

        // Generate EBO (Indices)
        for i in 0..latitudes {
            let mut k1 = i * (longtitudes + 1);
            let mut k2 = k1 + longtitudes + 1;

            for _ in 0..longtitudes {
                if i != 0 {
                    self.ebo.push(k1);
                    self.ebo.push(k2);
                    self.ebo.push(k1 + 1);
                }

                if i != (latitudes - 1) {
                    self.ebo.push(k1 + 1);
                    self.ebo.push(k2);
                    self.ebo.push(k2 + 1);
                }
                k1 += 1;
                k2 += 1;
            }
        }
    }
}
