use glam::{Vec2, Vec3};

use crate::graphics::triangle::{Color, RasterTriangle};

pub struct Rasterizer {
    pub width: usize,
    pub height: usize,
    pub frame_buff: Vec<Color>,
    pub depth_buff: Vec<f32>,
    pub triangles:  Vec<RasterTriangle>,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            frame_buff: vec![Color {r: 0, g: 0, b: 0}; width * height],
            depth_buff: vec![f32::INFINITY; width * height],
            triangles:  vec![],
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.frame_buff.resize(width * height, Color::new(0, 0, 0));
        self.depth_buff.resize(width * height, f32::INFINITY);
    }

    pub fn clear(&mut self) {
        self.frame_buff.fill(Color::BLACK);
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

    fn draw_pixel(&mut self, p: (usize,usize), z: f32, color: Color) {
        let pixel_index: usize = p.1 * self.width + p.0;
        if self.depth_buff[pixel_index] > z {
            self.depth_buff[pixel_index] = z;
            self.frame_buff[pixel_index] = color;
        }
    }

    pub fn render_triangle(&mut self, triangle: RasterTriangle) {
        let (a, b, c): (Vec3, Vec3, Vec3) = (
            triangle.a.pos,
            triangle.b.pos,
            triangle.c.pos,
        );

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
                let p: Vec2         = self.screen_to_ndc((i, j));
                if !triangle.is_inside(p) {
                    continue;
                }
                let p_inv_w: f32    = triangle.interpolate_inv_w(p);
                let z: f32          = triangle.interpolate_depth_with_inv_w(p, p_inv_w);
                let color: Color    = triangle.interpolate_color_with_inv_w(p, p_inv_w);
                self.draw_pixel((i, j), z, color);
            }
        }
    }
}
