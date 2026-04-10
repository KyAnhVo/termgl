use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::graphics::{
    mesh::{Mesh, MeshVertexIndices},
    options::ShadingMode,
    projection::Camera,
    shader::Shader,
    vertex::{RasterVertex, Vertex},
};

pub struct Rasterizer {
    pub width: usize,
    pub height: usize,
    pub frame_buff: Vec<Vec3>,
    pub depth_buff: Vec<f32>,
    pub background_color: Vec3,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize, background_color: Vec3) -> Self {
        Self {
            width,
            height,
            frame_buff: vec![background_color; width * height],
            depth_buff: vec![f32::INFINITY; width * height],
            background_color,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.frame_buff
            .resize(width * height, self.background_color);
        self.depth_buff.resize(width * height, f32::INFINITY);
    }

    pub fn clear(&mut self) {
        self.frame_buff.fill(self.background_color);
        self.depth_buff.fill(f32::INFINITY);
    }

    pub fn ndc_to_screen(&self, ndc: Vec3) -> (usize, usize) {
        // map [-1, 1] x [-1, 1] to [0, width - 1] x [height - 1, 0]
        let x: usize = ((ndc.x + 1.0) * 0.5 * (self.width as f32 - 1.0)).round() as usize;
        let y: usize = ((1.0 - ndc.y) * 0.5 * (self.height as f32 - 1.0)).round() as usize;
        (x, y)
    }

    pub fn screen_to_ndc(&self, screen_xy: (usize, usize)) -> Vec2 {
        // map [0, width - 1] x [height - 1, 0] to [-1, 1] x [-1, 1]

        // first, map to [0, 1] x [1, 0]. Add 0.5 to get into middle of pixel.
        let x_norm: f32 = (screen_xy.0 as f32 + 0.5) / self.width as f32;
        let y_norm: f32 = (screen_xy.1 as f32 + 0.5) / self.height as f32;

        // then map [0, 1] x [1, 0] to [-1, 1] x [-1, 1] by mult 2 sub 1
        let x: f32 = (x_norm * 2.0) - 1.0;
        let y: f32 = 1.0 - (y_norm * 2.0);

        Vec2::new(x, y)
    }

    fn draw_pixel(&mut self, p: (usize, usize), z: f32, color: Vec3) {
        let pixel_index: usize = p.1 * self.width + p.0;
        if self.depth_buff[pixel_index] > z {
            self.depth_buff[pixel_index] = z;
            self.frame_buff[pixel_index] = color;
        }
    }

    pub fn rasterize_mesh(
        &mut self,
        mesh: &Mesh,
        shader: &Shader,
        camera: Camera,
        shading_mode: ShadingMode,
    ) {
        for i in 0..(mesh.triangles.len() / 3) {
            self.rasterize_triangle(mesh, 3 * i, shader, camera, shading_mode);
        }
    }

    pub fn rasterize_triangle(
        &mut self,
        mesh: &Mesh,
        start_ind: usize,
        shader: &Shader,
        camera: Camera,
        shading_mode: ShadingMode,
    ) {
        let (ia, ib, ic): (MeshVertexIndices, MeshVertexIndices, MeshVertexIndices) = (
            mesh.triangles[start_ind],
            mesh.triangles[start_ind + 1],
            mesh.triangles[start_ind + 2],
        );
        let (va, vb, vc): (Vertex, Vertex, Vertex) = (
            mesh.vertices_world_space[ia.vertex_ind],
            mesh.vertices_world_space[ib.vertex_ind],
            mesh.vertices_world_space[ic.vertex_ind],
        );
        let (ra, rb, rc): (RasterVertex, RasterVertex, RasterVertex) = (
            mesh.raster_vertices[ia.vertex_ind],
            mesh.raster_vertices[ib.vertex_ind],
            mesh.raster_vertices[ic.vertex_ind],
        );
        let (na, nb, nc): (Vec4, Vec4, Vec4) = (
            mesh.normals_world_space[ia.normal_ind],
            mesh.normals_world_space[ib.normal_ind],
            mesh.normals_world_space[ic.normal_ind],
        );
        let (uva, uvb, uvc): (Vec2, Vec2, Vec2) =
            (mesh.uv[ia.uv_ind], mesh.uv[ib.uv_ind], mesh.uv[ic.uv_ind]);

        // these bunch for gouraud shading
        // tbf at this point just gouraud it if not phong
        // but oh well styling purpose I suppose :/
        let vertices_uv_parallaxed: (Vec2, Vec2, Vec2) = (
            mesh.get_parallaxed_uv(uva, &camera),
            mesh.get_parallaxed_uv(uvb, &camera),
            mesh.get_parallaxed_uv(uvc, &camera),
        );
        let vertices_kd: (Vec3, Vec3, Vec3) = match &mesh.texture_map {
            Some(texture_map) => (
                texture_map.interpolate(vertices_uv_parallaxed.0),
                texture_map.interpolate(vertices_uv_parallaxed.1),
                texture_map.interpolate(vertices_uv_parallaxed.2),
            ),
            None => (ra.color, rb.color, rc.color),
        };
        let vertices_shaded_color: (Vec3, Vec3, Vec3) = (
            shader.shade_point_phong(va.pos.xyz(), na, mesh.material, vertices_kd.0, camera),
            shader.shade_point_phong(vb.pos.xyz(), nb, mesh.material, vertices_kd.1, camera),
            shader.shade_point_phong(vc.pos.xyz(), nc, mesh.material, vertices_kd.2, camera),
        );

        if RasterVertex::is_back_facing(ra, rb, rc) {
            return;
        }

        if ra.pos.x.abs() > 1.0 || rb.pos.x.abs() > 1.0 || rc.pos.x.abs() > 1.0 {
            return;
        }
        if ra.pos.y.abs() > 1.0 || rb.pos.y.abs() > 1.0 || rc.pos.y.abs() > 1.0 {
            return;
        }

        let (a, b, c): (Vec3, Vec3, Vec3) = (ra.pos, rb.pos, rc.pos);

        let a_pix: (usize, usize) = self.ndc_to_screen(a);
        let b_pix: (usize, usize) = self.ndc_to_screen(b);
        let c_pix: (usize, usize) = self.ndc_to_screen(c);

        let vertices_x: Vec<usize> = vec![a_pix.0, b_pix.0, c_pix.0];
        let vertices_y: Vec<usize> = vec![a_pix.1, b_pix.1, c_pix.1];

        let min_x: usize = vertices_x[0].min(vertices_x[1]).min(vertices_x[2]);
        let max_x: usize = vertices_x[0].max(vertices_x[1]).max(vertices_x[2]);

        let min_y: usize = vertices_y[0].min(vertices_y[1]).min(vertices_y[2]);
        let max_y: usize = vertices_y[0].max(vertices_y[1]).max(vertices_y[2]);

        for i in min_x..=max_x {
            for j in min_y..=max_y {
                let p: Vec2 = self.screen_to_ndc((i, j));
                if !RasterVertex::is_inside(ra, rb, rc, p) {
                    continue;
                }

                // both used for later interpolations
                let barycentric_coordinate: (f32, f32, f32) =
                    RasterVertex::barycentric_coordinate(ra, rb, rc, p);
                let p_inv_w: f32 =
                    RasterVertex::interpolate_inv_w(ra, rb, rc, barycentric_coordinate);

                // z is used for depth buffering
                let z: f32 =
                    RasterVertex::interpolate_z((ra, rb, rc), barycentric_coordinate, p_inv_w);

                let color: Vec3 = if mesh.no_shade {
                    // just interpolate color over
                    RasterVertex::interpolate_color(
                        (ra, rb, rc),
                        (ra.color, rb.color, rc.color),
                        barycentric_coordinate,
                        p_inv_w,
                    )
                } else if shading_mode == ShadingMode::Phong {
                    // use phong shading
                    // interpolate uv, then parallax it with the height map
                    let uv: Vec2 = mesh.get_parallaxed_uv(
                        RasterVertex::interpolate_uv(
                            (ra, rb, rc),
                            (uva, uvb, uvc),
                            barycentric_coordinate,
                            p_inv_w,
                        ),
                        &camera,
                    );

                    // if there is no normal map, we interpolate the normals.
                    // else we use the interpolated and parallaxed uv to
                    // calculate the normal.
                    let n: Vec4 = match &mesh.normal_map {
                        Some(normal_map) => normal_map
                            .interpolate(
                                uv,
                                (va.pos.xyz(), vb.pos.xyz(), vc.pos.xyz()),
                                (uva, uvb, uvc),
                            )
                            .extend(0.0),
                        None => RasterVertex::interpolate_normals(
                            (ra, rb, rc),
                            (na, nb, nc),
                            barycentric_coordinate,
                            p_inv_w,
                        )
                        .normalize(),
                    };

                    // if texture map exists, we use our uv to get from texture map,
                    // else we interpolate color.
                    let kd: Vec3 = match &mesh.texture_map {
                        Some(texture_map) => texture_map.interpolate(uv),
                        None => RasterVertex::interpolate_color(
                            (ra, rb, rc),
                            (ra.color, rb.color, rc.color),
                            barycentric_coordinate,
                            p_inv_w,
                        ),
                    };

                    // interpolate position... no uv here sorry hehe
                    let pos: Vec3 = RasterVertex::interpolate_position(
                        (ra, rb, rc),
                        (va.pos, vb.pos, vc.pos),
                        barycentric_coordinate,
                        p_inv_w,
                    )
                    .xyz();

                    // probably goes well, no crash
                    shader.shade_point_phong(pos, n, mesh.material, kd, camera)
                } else if shading_mode == ShadingMode::Gouraud {
                    // gouraud shading: interpolate color from vertices
                    RasterVertex::interpolate_color(
                        (ra, rb, rc),
                        (
                            vertices_shaded_color.0,
                            vertices_shaded_color.1,
                            vertices_shaded_color.2,
                        ),
                        barycentric_coordinate,
                        p_inv_w,
                    )
                } else {
                    // flat shading: use the first vertex's color
                    vertices_shaded_color.0
                };

                self.draw_pixel((i, j), z, color);
            }
        }
    }
}
