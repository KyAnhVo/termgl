use crate::graphics::vertex::{Material, RasterVertex, Vertex};
use glam::{Mat3, Vec3, Vec4, Vec4Swizzles};

/// An object to be rendered, represented by an EBO and a VAO
pub struct Mesh {
    /// The origin of the mesh in object space
    pub origin: Vec3,

    /// The object space's orthonormal basis
    pub orthonormal_basis: Mat3,

    /// The vertices in the VAO is in object space
    pub vao: Vec<Vertex>,

    /// Each element in the EBO maps to a vertex in
    /// the VAO, and each 3 elements 3x, 3x+1, 3x+2 in
    /// the EBO creates a triangle.
    /// Note: ebo.size() % 3 == 0 must remain true.
    pub ebo: Vec<usize>,

    /// Used for RasterVertex after projection
    pub projected_vao: Vec<RasterVertex>,

    /// Used for Gouraud and Phong shading. Before calling
    /// self.finalize_normals, the orthogonals are not normalized.
    pub vertex_orthogonals: Vec<Vec4>,

    /// As of this version, expect the mesh to remain the same material
    pub material: Material,

    /// Some objects might not want to be shaded (e.g. background image,
    /// 2D game with no shading, or a light source)
    pub no_shade: bool,

    /// After calling finalize_normals, the orthogonals are normalized and
    /// the mesh is ready for shading. No further changes to the mesh should
    /// be made after this point.
    pub finalized: bool,
}

impl Mesh {
    pub fn new(origin: Vec3, orthonormal_basis: Mat3, material: Material, no_shade: bool) -> Self {
        Self {
            origin,
            orthonormal_basis,
            vao: vec![],
            ebo: vec![],
            projected_vao: vec![],
            vertex_orthogonals: vec![],
            material,
            no_shade,
            finalized: false,
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
        for i in 0..self.vertex_orthogonals.len() {
            if self.vertex_orthogonals[i] == Vec4::ZERO {
                continue;
            }
            self.vertex_orthogonals[i] = self.vertex_orthogonals[i].normalize();
        }
        self.finalized = true;
    }
}
