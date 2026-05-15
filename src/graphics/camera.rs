use crossterm::terminal;
use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

/// A pinhole camera defined by its position and orientation in world space.
///
/// Stores the view basis (up, gaze, right) as Vec4 directions (w = 0) and the
/// position as a Vec4 point (w = 1). Aspect ratio is kept in sync with the
/// terminal size via [`Camera::resize`].
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
    /// Creates a camera from a reference up direction, a gaze direction, a world-space
    /// position, and a vertical field of view in radians.
    ///
    /// The basis is re-orthonormalized, so the provided vectors need not be perfectly
    /// orthogonal. Aspect ratio is initialized from the current terminal size.
    pub fn new(up: Vec3, gaze: Vec3, pos: Vec3, fov: f32) -> Self {
        let mut up3: Vec3 = up.normalize();
        let gaze3: Vec3 = gaze.normalize();
        let right3: Vec3 = up3.cross(gaze3);
        up3 = gaze3.cross(right3).normalize();

        let (width_u16, height_u16) = terminal::size().unwrap();
        let (width, height) = (width_u16 as usize, height_u16 as usize * 2);
        let aspect_ratio: f32 = width as f32 / height as f32;

        Self {
            up: up3.extend(0.0),
            gaze: gaze3.extend(0.0),
            right: right3.extend(0.0),
            pos: pos.extend(1.0),
            fov,
            aspect_ratio,
        }
    }

    /// Reorients the camera to look at `at` from `from`, using `up` as the reference
    /// up direction. Also updates the camera's position to `from`.
    pub fn look_at(&mut self, at: Vec3, from: Vec3, up: Vec3) {
        let mut up3: Vec3 = up.normalize();
        let gaze3: Vec3 = (at - from).normalize();
        let right3: Vec3 = up3.cross(gaze3).normalize();
        up3 = gaze3.cross(right3).normalize();

        self.up = up3.extend(0.0);
        self.right = right3.extend(0.0);
        self.gaze = gaze3.extend(0.0);
        self.pos = from.extend(1.0);
    }

    /// Updates the aspect ratio to match a new terminal size. Call this on terminal resize events.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.aspect_ratio = width as f32 / (height as f32);
    }

    /// Returns the view matrix that transforms world-space coordinates into camera space.
    pub fn m_view(self) -> Mat4 {
        let t_view: Mat4 = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(-self.pos.x, -self.pos.y, -self.pos.z, 1.0),
        );
        let r_view: Mat4 = Mat4::from_cols(
            Vec4::new(self.right.x, self.up.x, self.gaze.x, 0.0),
            Vec4::new(self.right.y, self.up.y, self.gaze.y, 0.0),
            Vec4::new(self.right.z, self.up.z, self.gaze.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        r_view * t_view
    }

    /// Returns the perspective projection matrix for near plane `n` and far plane `f`.
    ///
    /// Derives the frustum bounds from the camera's fov and aspect ratio, then composes
    /// the perspective-to-ortho squish with [`Camera::m_ortho`].
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

    /// Returns an orthographic projection matrix mapping the axis-aligned box
    /// [`l`, `r`] × [`b`, `t`] × [`n`, `f`] to the canonical NDC cube [-1, 1]^3.
    pub(crate) fn m_ortho(self, l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4 {
        let m_ortho_s: Mat4 = Mat4::from_cols(
            Vec4::new(2.0 / (r - l), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / (t - b), 0.0, 0.0),
            Vec4::new(0.0, 0.0, 2.0 / (f - n), 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        let m_ortho_t: Mat4 = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(-(r + l) / 2.0, -(t + b) / 2.0, -(n + f) / 2.0, 1.0),
        );
        m_ortho_s * m_ortho_t
    }
}
