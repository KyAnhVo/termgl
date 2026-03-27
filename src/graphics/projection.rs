use glam::{
    Vec4, Vec4Swizzles, Vec3,
    Mat4,
};

#[derive(Clone, Copy)]
pub struct Camera {

    /// up vector
    pub t: Vec4,

    /// gaze vector
    pub g: Vec4,

    /// right vector
    pub r: Vec4,

    /// position of camera
    pub e: Vec4,
    
    /// field of view (in radians)
    fov: f32,

    /// aspect ratio (height / width of screen)
    aspect_ratio: f32,
}

impl Camera {
    pub fn new(up: Vec4, gaze: Vec4, pos: Vec4, fov: f32, aspect_ratio: f32) -> Self {
        if up.w != 0.0 || gaze.w != 0.0 {
            panic!("up ({}, {}, {}, {}) and gaze({}, {}, {}, {}) are direction, not position",
                up.x, up.y, up.z, up.w,
                gaze.x, gaze.y, gaze.z, gaze.w
            );
        }
        if pos.w != 1.0 {
            panic!("pos ({}, {}, {}, {}) must have w value 1.0f32",
                pos.x, pos.y, pos.z, pos.w
            );
        }


        let mut up3: Vec3   = up.xyz().normalize();
        let gaze3: Vec3     = gaze.xyz().normalize();
        let right3: Vec3    = gaze3.cross(up3);
        up3                 = right3.cross(gaze3).normalize();

        Self {
            t: up3.extend(0.0),
            g: gaze3.extend(0.0),
            r: right3.extend(0.0),
            e: pos,
            fov,
            aspect_ratio,
        }
    }

    pub fn m_view(self) -> Mat4 {
        let t_view: Mat4 = Mat4::from_cols(
            Vec4::new(1.0,          0.0,        0.0,        0.0),
            Vec4::new(0.0,          1.0,        0.0,        0.0),
            Vec4::new(0.0,          0.0,        1.0,        0.0),
            Vec4::new(-self.e.x,    -self.e.y,  -self.e.z,  1.0),
        );
        let r_view: Mat4 = Mat4::from_cols(
            Vec4::new(self.r.x,     self.t.x,   -self.g.x,  0.0),
            Vec4::new(self.r.y,     self.t.y,   -self.g.y,  0.0),
            Vec4::new(self.r.z,     self.t.z,   -self.g.z,  0.0),
            Vec4::new(0.0,          0.0,        0.0,        1.0),
        );
        r_view * t_view
    }

    pub fn m_perspective(self, n: f32, f: f32) -> Mat4 {
        let m_persp_to_ortho: Mat4 = Mat4::from_cols(
            Vec4::new(n,    0.0,    0.0,    0.0),
            Vec4::new(0.0,  n,      0.0,    0.0),
            Vec4::new(0.0,  0.0,    n + f,  1.0),
            Vec4::new(0.0,  0.0,    -n * f, 0.0),
        );

        self.m_ortho(n, f) * m_persp_to_ortho
    }

    pub fn m_ortho(self, n: f32, f: f32) -> Mat4 {
        let t: f32 = self.fov.tan() / 2.0 * n.abs();
        let r: f32 = t * self.aspect_ratio;
        let b: f32 = -t;
        let l: f32 = -r;

        let m_ortho_s: Mat4 = Mat4::from_cols(
            Vec4::new(2.0 / (r - l),    0.0,            0.0,            0.0),
            Vec4::new(0.0,              2.0 / (t - b),  0.0,            0.0), 
            Vec4::new(0.0,              0.0,            2.0 / (n - f),  0.0), 
            Vec4::new(0.0,              0.0,            0.0,            1.0),
        );
        let m_ortho_t: Mat4 = Mat4::from_cols(
            Vec4::new(1.0,  0.0,    0.0,                0.0),
            Vec4::new(0.0,  1.0,    0.0,                0.0),
            Vec4::new(0.0,  0.0,    1.0,                0.0),
            Vec4::new(0.0,  0.0,    -(n + f) / 2.0,     1.0),
        );

        m_ortho_s * m_ortho_t
    }
}
