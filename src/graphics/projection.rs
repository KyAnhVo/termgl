use crossterm::terminal;
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

#[derive(Clone, Copy)]
pub struct Camera {
    /// up vector
    pub up: Vec4,

    /// gaze vector
    pub gaze: Vec4,

    /// right vector
    pub right: Vec4,

    /// position of camera
    pub pos: Vec4,

    /// field of view (in radians)
    fov: f32,

    /// aspect ratio (height / width of screen)
    aspect_ratio: f32,
}

impl Camera {
    pub fn new(up: Vec4, gaze: Vec4, pos: Vec4, fov: f32) -> Self {
        if up.w != 0.0 || gaze.w != 0.0 {
            panic!(
                "up ({}, {}, {}, {}) and gaze({}, {}, {}, {}) are direction, not position",
                up.x, up.y, up.z, up.w, gaze.x, gaze.y, gaze.z, gaze.w
            );
        }
        if pos.w != 1.0 {
            panic!(
                "pos ({}, {}, {}, {}) must have w value 1.0f32",
                pos.x, pos.y, pos.z, pos.w
            );
        }

        let mut up3: Vec3 = up.xyz().normalize();
        let gaze3: Vec3 = gaze.xyz().normalize();
        let right3: Vec3 = gaze3.cross(up3);
        up3 = right3.cross(gaze3).normalize();

        let (width_u16, height_u16) = terminal::size().unwrap();
        let (width, height) = (width_u16 as usize, height_u16 as usize * 2);
        let aspect_ratio: f32 = width as f32 / height as f32;

        Self {
            up: up3.extend(0.0),
            gaze: gaze3.extend(0.0),
            right: right3.extend(0.0),
            pos,
            fov,
            aspect_ratio,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.aspect_ratio = width as f32 / (height as f32);
    }

    pub fn m_view(self) -> Mat4 {
        let t_view: Mat4 = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(-self.pos.x, -self.pos.y, -self.pos.z, 1.0),
        );
        let r_view: Mat4 = Mat4::from_cols(
            Vec4::new(self.right.x, self.up.x, -self.gaze.x, 0.0),
            Vec4::new(self.right.y, self.up.y, -self.gaze.y, 0.0),
            Vec4::new(self.right.z, self.up.z, -self.gaze.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        r_view * t_view
    }

    pub fn m_perspective(self, n: f32, f: f32) -> Mat4 {
        let m_persp_to_ortho: Mat4 = Mat4::from_cols(
            Vec4::new(n, 0.0, 0.0, 0.0),
            Vec4::new(0.0, n, 0.0, 0.0),
            Vec4::new(0.0, 0.0, n + f, 1.0),
            Vec4::new(0.0, 0.0, -n * f, 0.0),
        );

        let t: f32 = self.fov.tan() / 2.0 * n.abs();
        let r: f32 = t * self.aspect_ratio;
        let b: f32 = -t;
        let l: f32 = -r;

        let m_ortho: Mat4 = self.m_ortho(l, r, b, t, n, f);
        m_ortho * m_persp_to_ortho
    }

    pub fn m_ortho(self, l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4 {
        let m_ortho_s: Mat4 = Mat4::from_cols(
            Vec4::new(2.0 / (r - l), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / (t - b), 0.0, 0.0),
            Vec4::new(0.0, 0.0, 2.0 / (n - f), 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        let m_ortho_t: Mat4 = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, -(n + f) / 2.0, 1.0),
        );
        m_ortho_s * m_ortho_t
    }
}
