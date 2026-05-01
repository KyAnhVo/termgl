use crate::graphics::{Material, Mesh, Vertex, VertexIndices};
use glam::{Vec3, Vec4, Vec4Swizzles};
use std::{collections::HashSet, f32, rc::Rc};

/// Implements the vertex cluster simplification algorithm by Rossignac and Borrel.
///
/// Note: do not use this if your mesh is not a manifold mesh.
pub fn vertex_cluster(mesh: &Mesh, hxyz: f32) -> Mesh {
    let mut simplified_mesh = Mesh::new(mesh.material, mesh.no_shade);
    simplified_mesh.default_color = mesh.default_color;

    // TODO: Implement vertex cluster simplification algorithm

    // 1. Get vertex grade
    let vertex_edge_mapping: Rc<[Rc<[usize]>]> = get_vertex_edges_mapping(mesh);
    let vertex_grades: Rc<[f32]> = get_vertex_grades(mesh, &vertex_edge_mapping);

    // 2. Get the volume to be bounded, and subsequently the bounding box count
    // on each dimension
    let (mut x_min, mut x_max): (f32, f32) = (0.0, 0.0);
    let (mut y_min, mut y_max): (f32, f32) = (0.0, 0.0);
    let (mut z_min, mut z_max): (f32, f32) = (0.0, 0.0);
    for v in mesh.vertices.iter() {
        let v_vec3: Vec3 = v.pos.xyz() / v.pos.w;
        let (x, y, z): (f32, f32, f32) = (v_vec3.x, v_vec3.y, v_vec3.z);
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
        z_min = z_min.min(z);
        z_max = z_max.max(z);
    }
    let x_dim: usize = ((x_max - x_min) / hxyz).floor() as usize + 1;
    let y_dim: usize = ((y_max - y_min) / hxyz).floor() as usize + 1;
    let z_dim: usize = ((z_max - z_min) / hxyz).floor() as usize + 1;

    let mut cells: Cells = Cells::new(
        (x_dim, y_dim, z_dim),
        hxyz,
        (x_min, x_max, y_min, y_max, z_min, z_max),
    );

    // 3. Calculate COM of each cell

    simplified_mesh
}

struct Cells {
    pub dim: (usize, usize, usize),
    pub x_bound: (f32, f32),
    pub y_bound: (f32, f32),
    pub z_bound: (f32, f32),
    pub hxyz: f32,
    pub cluster_centers: Vec<Vec3>,
    pub cluster_sizes: Vec<usize>,
}

impl Cells {
    fn new(
        dim: (usize, usize, usize),
        hxyz: f32,
        bounding_box: (f32, f32, f32, f32, f32, f32),
    ) -> Self {
        let arr_size: usize = dim.0 * dim.1 * dim.2;
        Self {
            dim,
            x_bound: (bounding_box.0, bounding_box.1),
            y_bound: (bounding_box.2, bounding_box.3),
            z_bound: (bounding_box.4, bounding_box.5),
            hxyz,
            cluster_centers: vec![Vec3::ZERO; arr_size],
            cluster_sizes: vec![0; arr_size],
        }
    }

    fn index_of(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.dim.0 + z * self.dim.0 * self.dim.1
    }

    fn at(&self, vertex: Vec3) -> (usize, usize, usize) {
        let (x, y, z): (f32, f32, f32) = (vertex.x, vertex.y, vertex.z);
        let x_box: usize = ((x - self.x_bound.0) / self.hxyz).floor() as usize;
        let y_box: usize = ((y - self.y_bound.0) / self.hxyz).floor() as usize;
        let z_box: usize = ((z - self.z_bound.0) / self.hxyz).floor() as usize;
        (x_box, y_box, z_box)
    }
}

fn get_vertex_edges_mapping(mesh: &Mesh) -> Rc<[Rc<[usize]>]> {
    let mut mapping: Vec<Vec<usize>> = vec![vec![]; mesh.vertices.len()];
    for triangle in mesh.triangles.chunks_exact(3) {
        let (a, b, c): (usize, usize, usize) = (
            triangle[0].vertex_ind,
            triangle[1].vertex_ind,
            triangle[2].vertex_ind,
        );
        mapping[a].push(b);
        mapping[a].push(c);
        mapping[b].push(a);
        mapping[b].push(c);
        mapping[c].push(a);
        mapping[c].push(b);
    }

    for neighbors in &mut mapping {
        neighbors.sort_unstable();
        neighbors.dedup();
    }

    mapping
        .into_iter()
        .map(Rc::<[usize]>::from)
        .collect::<Rc<[Rc<[usize]>]>>()
}

/// Calculate grades for vertices
///
/// Vertices are graded higher if
/// vertices have higher chance of lying on the object's silhouettes from arbitrary POV or
/// vertices bound large faces
fn get_vertex_grades(mesh: &Mesh, edges: &[Rc<[usize]>]) -> Rc<[f32]> {
    let mut grades: Vec<f32> = vec![];
    for (i, neighbors) in edges.iter().enumerate() {
        let my_vertex: Vec3 = mesh.vertices[i].pos.xyz() / mesh.vertices[i].pos.w;
        if neighbors.len() == 0 {
            grades.push(0.0);
            continue;
        }
        // Calculate max edge length
        let mut max_edge_len: f32 = 0.0;
        for j in neighbors.iter() {
            let neighbor_vertex: Vec3 = mesh.vertices[*j].pos.xyz() / mesh.vertices[*j].pos.w;
            let edge_len = (my_vertex - neighbor_vertex).length();
            if edge_len > max_edge_len {
                max_edge_len = edge_len;
            }
        }

        // Calculate max |cosine(theta)| where theta is angle between all pairs of neighbors
        let mut max_cos: f32 = 0.0;
        for j in 0..(neighbors.len() - 1) {
            let vertex_j: Vec3 =
                mesh.vertices[neighbors[j]].pos.xyz() / mesh.vertices[neighbors[j]].pos.w;
            let edge_ij: Vec3 = (vertex_j - my_vertex).normalize();
            for k in (j + 1)..neighbors.len() {
                let vertex_k: Vec3 =
                    mesh.vertices[neighbors[k]].pos.xyz() / mesh.vertices[neighbors[k]].pos.w;
                let edge_ik: Vec3 = (vertex_k - my_vertex).normalize();
                let cosine: f32 = edge_ij.dot(edge_ik).abs();
                max_cos = max_cos.max(cosine);
            }
        }

        grades.push((1.0 - max_cos) * max_edge_len)
    }
    Rc::from(grades)
}
