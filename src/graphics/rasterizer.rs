use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::graphics::{
    UVMap,
    camera::Camera,
    mesh::{Mesh, VertexIndices},
    options::ShadingMode,
    shader::Shader,
    vertex::{RasterVertex, Vertex},
};

/// The background to be rasterized underneath any scene.
#[derive(Clone, PartialEq)]
pub enum Background {
    /// Just a solid color as background
    SolidColor(Vec3),
    /// Gradient starts from (0, 0).
    /// Any two points s.t. y-x == direction also has the same color
    Gradient {
        color1: Vec3,
        color2: Vec3,
        direction: (f32, f32),
    },
    /// uses an image with uv map, the screen's uv depends on uv0 (top left), and the
    /// rectangle defined by topleft (uv0), u_range (width), v_range (height).
    Image {
        img: UVMap,
        u_range: f32,
        v_range: f32,
        uv0: Vec2,
    },
}

struct BackgroundCache {
    background: Option<Background>,
    buf: Vec<Vec3>,
}

pub(crate) struct Rasterizer {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) frame_buff: Vec<Vec3>,
    pub(crate) depth_buff: Vec<f32>,
    background_cache: BackgroundCache,
}

impl Rasterizer {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            frame_buff: vec![Vec3::ZERO; width * height],
            depth_buff: vec![f32::INFINITY; width * height],
            background_cache: BackgroundCache {
                background: None,
                buf: vec![Vec3::ZERO; width * height],
            },
        }
    }

    pub(crate) fn resize(&mut self, width: usize, height: usize) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        self.frame_buff.resize(width * height, Vec3::ZERO);
        self.depth_buff.resize(width * height, f32::INFINITY);
        self.background_cache.background = None;
    }

    pub(crate) fn clear(&mut self, background: &Background) {
        match &self.background_cache.background {
            None => {}
            Some(cache) => {
                if cache == background {
                    self.frame_buff.clone_from(&self.background_cache.buf);
                    self.depth_buff.fill(f32::INFINITY);
                    return;
                }
            }
        }

        match background {
            Background::SolidColor(val) => {
                self.frame_buff.fill(*val);
                self.depth_buff.fill(f32::INFINITY);
            }
            Background::Image {
                img,
                u_range,
                v_range,
                uv0,
            } => {
                for i in 0..self.width {
                    for j in 0..self.height {
                        // from uv0, get screen's [0, 1] range, then stretch it by (u_range, v_range)
                        let uv: Vec2 = uv0
                            + Vec2::new(
                                (i as f32 + 0.5) / self.width as f32,
                                (j as f32 + 0.5) / self.height as f32,
                            ) * Vec2::new(*u_range, *v_range);
                        let color: Vec3 = img.interpolate(uv);
                        self.draw_force((i, j), f32::INFINITY, color);
                    }
                }
            }
            Background::Gradient {
                color1,
                color2,
                direction: (dx, dy),
            } => {
                let w = self.width.max(1) as f32;
                let h = self.height.max(1) as f32;
                for j in 0..self.height {
                    for i in 0..self.width {
                        let t = (i as f32 / w * dx + j as f32 / h * dy).clamp(0.0, 1.0);
                        let color = *color1 * (1.0 - t) + *color2 * t;
                        self.draw_force((i, j), f32::INFINITY, color);
                    }
                }
            }
        }
        self.background_cache.background = Some(background.clone());
        self.background_cache.buf.clone_from(&self.frame_buff);
    }

    fn ndc_to_screen(&self, ndc: Vec3) -> (isize, isize) {
        // map [-1, 1] x [-1, 1] to [0, width - 1] x [height - 1, 0]
        let x: isize = ((ndc.x + 1.0) * 0.5 * (self.width as f32 - 1.0)).round() as isize;
        let y: isize = ((1.0 - ndc.y) * 0.5 * (self.height as f32 - 1.0)).round() as isize;
        (x, y)
    }

    fn screen_to_ndc(&self, screen_xy: (usize, usize)) -> Vec2 {
        // map [0, width - 1] x [height - 1, 0] to [-1, 1] x [-1, 1]

        // first, map to [0, 1] x [1, 0]. Add 0.5 to get into middle of pixel.
        let x_norm: f32 = (screen_xy.0 as f32 + 0.5) / self.width as f32;
        let y_norm: f32 = (screen_xy.1 as f32 + 0.5) / self.height as f32;

        // then map [0, 1] x [1, 0] to [-1, 1] x [-1, 1] by multiply 2 sub 1
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

    fn draw_force(&mut self, p: (usize, usize), z: f32, color: Vec3) {
        let pixel_index: usize = p.1 * self.width + p.0;
        self.depth_buff[pixel_index] = z;
        self.frame_buff[pixel_index] = color;
    }

    pub(crate) fn rasterize_mesh(
        &mut self,
        mesh: &mut Mesh,
        shader: &Shader,
        camera: &Camera,
        shading_mode: ShadingMode,
    ) {
        mesh.finalize_mesh();
        for i in 0..(mesh.triangles.len() / 3) {
            self.rasterize_triangle(mesh, 3 * i, shader, camera, shading_mode);
        }
    }

    fn rasterize_triangle(
        &mut self,
        mesh: &Mesh,
        start_ind: usize,
        shader: &Shader,
        camera: &Camera,
        shading_mode: ShadingMode,
    ) {
        match shading_mode {
            ShadingMode::Phong => self.rasterize_triangle_phong(mesh, start_ind, shader, camera),
            ShadingMode::Gouraud => {
                self.rasterize_triangle_gouraud(mesh, start_ind, shader, camera)
            }
        }
    }

    fn get_rendering_rectangle(
        &mut self,
        a: Vec3,
        b: Vec3,
        c: Vec3,
    ) -> (usize, usize, usize, usize) {
        let a_pix: (isize, isize) = self.ndc_to_screen(a);
        let b_pix: (isize, isize) = self.ndc_to_screen(b);
        let c_pix: (isize, isize) = self.ndc_to_screen(c);

        let min_x: usize = (a_pix.0.min(b_pix.0).min(c_pix.0).max(0) as usize).min(self.width - 1);
        let max_x: usize = (a_pix.0.max(b_pix.0).max(c_pix.0).max(0) as usize).min(self.width - 1);
        let min_y: usize = (a_pix.1.min(b_pix.1).min(c_pix.1).max(0) as usize).min(self.height - 1);
        let max_y: usize = (a_pix.1.max(b_pix.1).max(c_pix.1).max(0) as usize).min(self.height - 1);
        (min_x, max_x, min_y, max_y)
    }

    fn get_triangle_vertex_values(
        mesh: &Mesh,
        start_ind: usize,
    ) -> (
        (Vertex, Vertex, Vertex),
        (RasterVertex, RasterVertex, RasterVertex),
        (Vec4, Vec4, Vec4),
        (Vec2, Vec2, Vec2),
    ) {
        let (ia, ib, ic): (VertexIndices, VertexIndices, VertexIndices) = (
            mesh.triangles[start_ind],
            mesh.triangles[start_ind + 1],
            mesh.triangles[start_ind + 2],
        );
        let vertices: (Vertex, Vertex, Vertex) = (
            mesh.vertices_world_space[ia.vertex_ind],
            mesh.vertices_world_space[ib.vertex_ind],
            mesh.vertices_world_space[ic.vertex_ind],
        );
        let raster_vertices: (RasterVertex, RasterVertex, RasterVertex) = (
            mesh.raster_vertices[ia.vertex_ind],
            mesh.raster_vertices[ib.vertex_ind],
            mesh.raster_vertices[ic.vertex_ind],
        );
        let normals: (Vec4, Vec4, Vec4) = (
            mesh.normals_world_space[ia.normal_ind],
            mesh.normals_world_space[ib.normal_ind],
            mesh.normals_world_space[ic.normal_ind],
        );
        let uvs: (Vec2, Vec2, Vec2) = (mesh.uv[ia.uv_ind], mesh.uv[ib.uv_ind], mesh.uv[ic.uv_ind]);
        (vertices, raster_vertices, normals, uvs)
    }

    fn rasterize_triangle_phong(
        &mut self,
        mesh: &Mesh,
        start_ind: usize,
        shader: &Shader,
        camera: &Camera,
    ) {
        let (vertices, raster_vertices, normals, uvs) =
            Self::get_triangle_vertex_values(mesh, start_ind);

        if Self::triangle_is_outside_camera(raster_vertices) {
            return;
        }

        let (min_x, max_x, min_y, max_y) = self.get_rendering_rectangle(
            raster_vertices.0.pos,
            raster_vertices.1.pos,
            raster_vertices.2.pos,
        );

        for i in min_x..=max_x {
            for j in min_y..=max_y {
                let pos: Vec2 = self.screen_to_ndc((i, j));
                let barycentric_coordinate = RasterVertex::barycentric_coordinate(
                    raster_vertices.0,
                    raster_vertices.1,
                    raster_vertices.2,
                    pos,
                );
                if barycentric_coordinate.0 < 0.0
                    || barycentric_coordinate.1 < 0.0
                    || barycentric_coordinate.2 < 0.0
                {
                    continue;
                }
                let p_inv_w: f32 = RasterVertex::interpolate_inv_w(
                    raster_vertices.0,
                    raster_vertices.1,
                    raster_vertices.2,
                    barycentric_coordinate,
                );
                let z: f32 =
                    RasterVertex::interpolate_z(raster_vertices, barycentric_coordinate, p_inv_w);
                let color = Self::shade_pixel_phong(
                    &mesh,
                    shader,
                    camera,
                    vertices,
                    raster_vertices,
                    normals,
                    uvs,
                    barycentric_coordinate,
                    p_inv_w,
                );
                self.draw_pixel((i, j), z, color);
            }
        }
    }

    /// Shade pixel for phong shading
    fn shade_pixel_phong(
        mesh: &&Mesh,
        shader: &Shader,
        camera: &Camera,
        vertices: (Vertex, Vertex, Vertex),
        raster_vertices: (RasterVertex, RasterVertex, RasterVertex),
        normals: (Vec4, Vec4, Vec4),
        uvs: (Vec2, Vec2, Vec2),
        barycentric_coordinate: (f32, f32, f32),
        p_inv_w: f32,
    ) -> Vec3 {
        let uv: Vec2 = mesh.get_parallax_uv(
            RasterVertex::interpolate_uv(raster_vertices, uvs, barycentric_coordinate, p_inv_w),
            &camera,
        );
        // if there is no normal map, we interpolate the normals.
        // else we use the interpolated and parallax uv to
        // calculate the normal.
        let n: Vec4 = match &mesh.normal_map {
            Some(normal_map) => normal_map
                .interpolate(
                    uv,
                    (
                        vertices.0.pos.xyz(),
                        vertices.1.pos.xyz(),
                        vertices.2.pos.xyz(),
                    ),
                    uvs,
                )
                .extend(0.0),
            None => RasterVertex::interpolate_normals(
                raster_vertices,
                normals,
                barycentric_coordinate,
                p_inv_w,
            )
            .normalize(),
        };

        // if texture map exists, we use our uv to get from texture map,
        // else we interpolate color.
        let kd: Vec3 = match &mesh.texture_map {
            Some(texture_map) => texture_map.interpolate(uv),
            None => mesh.material.diffuse_constant,
        };

        let pos: Vec3 = RasterVertex::interpolate_position(
            raster_vertices,
            (vertices.0.pos, vertices.1.pos, vertices.2.pos),
            barycentric_coordinate,
            p_inv_w,
        )
        .xyz();

        if mesh.no_shade {
            kd
        } else {
            shader.shade_point_phong(pos, n, mesh.material, kd, camera)
        }
    }

    fn rasterize_triangle_gouraud(
        &mut self,
        mesh: &Mesh,
        start_ind: usize,
        shader: &Shader,
        camera: &Camera,
    ) {
        let (vertices, raster_vertices, normals, uvs) =
            Self::get_triangle_vertex_values(mesh, start_ind);

        if Self::triangle_is_outside_camera(raster_vertices) {
            return;
        }

        let vertex_colors: (Vec3, Vec3, Vec3) = Self::shade_vertices_gouraud(
            &mesh,
            shader,
            &camera,
            vertices,
            raster_vertices,
            normals,
            uvs,
        );

        let (min_x, max_x, min_y, max_y) = self.get_rendering_rectangle(
            raster_vertices.0.pos,
            raster_vertices.1.pos,
            raster_vertices.2.pos,
        );

        for i in min_x..=max_x {
            for j in min_y..=max_y {
                let pos: Vec2 = self.screen_to_ndc((i, j));
                let barycentric_coordinate = RasterVertex::barycentric_coordinate(
                    raster_vertices.0,
                    raster_vertices.1,
                    raster_vertices.2,
                    pos,
                );
                if barycentric_coordinate.0 < 0.0
                    || barycentric_coordinate.1 < 0.0
                    || barycentric_coordinate.2 < 0.0
                {
                    continue;
                }
                let p_inv_w = RasterVertex::interpolate_inv_w(
                    raster_vertices.0,
                    raster_vertices.1,
                    raster_vertices.2,
                    barycentric_coordinate,
                );
                let z: f32 =
                    RasterVertex::interpolate_z(raster_vertices, barycentric_coordinate, p_inv_w);
                let color: Vec3 = RasterVertex::interpolate_color(
                    raster_vertices,
                    vertex_colors,
                    barycentric_coordinate,
                    p_inv_w,
                );
                self.draw_pixel((i, j), z, color);
            }
        }
    }

    /// Shade the vertices for Gouraud shading
    fn shade_vertices_gouraud(
        mesh: &&Mesh,
        shader: &Shader,
        camera: &Camera,
        vertices: (Vertex, Vertex, Vertex),
        _raster_vertices: (RasterVertex, RasterVertex, RasterVertex),
        normals: (Vec4, Vec4, Vec4),
        uvs: (Vec2, Vec2, Vec2),
    ) -> (Vec3, Vec3, Vec3) {
        let uvs_parallax: (Vec2, Vec2, Vec2) = (
            mesh.get_parallax_uv(uvs.0, &camera),
            mesh.get_parallax_uv(uvs.1, &camera),
            mesh.get_parallax_uv(uvs.2, &camera),
        );
        let vertices_kd: (Vec3, Vec3, Vec3) = match &mesh.texture_map {
            Some(texture_map) => (
                texture_map.interpolate(uvs_parallax.0),
                texture_map.interpolate(uvs_parallax.1),
                texture_map.interpolate(uvs_parallax.2),
            ),
            None => (
                mesh.material.diffuse_constant,
                mesh.material.diffuse_constant,
                mesh.material.diffuse_constant,
            ),
        };
        if mesh.no_shade {
            return vertices_kd;
        }
        let vertices_shaded_color: (Vec3, Vec3, Vec3) = (
            shader.shade_point_phong(
                vertices.0.pos.xyz(),
                normals.0,
                mesh.material,
                vertices_kd.0,
                camera,
            ),
            shader.shade_point_phong(
                vertices.1.pos.xyz(),
                normals.1,
                mesh.material,
                vertices_kd.1,
                camera,
            ),
            shader.shade_point_phong(
                vertices.2.pos.xyz(),
                normals.2,
                mesh.material,
                vertices_kd.2,
                camera,
            ),
        );
        vertices_shaded_color
    }

    /// uses for early halting
    fn triangle_is_outside_camera(
        raster_vertices: (RasterVertex, RasterVertex, RasterVertex),
    ) -> bool {
        /*
        if RasterVertex::is_back_facing(raster_vertices.0, raster_vertices.1, raster_vertices.2) {
            return true;
        }
        */

        if raster_vertices.0.pos.x.abs() > 1.0
            || raster_vertices.1.pos.x.abs() > 1.0
            || raster_vertices.2.pos.x.abs() > 1.0
        {
            return true;
        }
        if raster_vertices.0.pos.y.abs() > 1.0
            || raster_vertices.1.pos.y.abs() > 1.0
            || raster_vertices.2.pos.y.abs() > 1.0
        {
            return true;
        }
        false
    }
}
