use std::ops::{Add, Div, Mul};

use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

#[derive(Clone, Copy)]
pub struct Material {
    pub ks: Vec3,
    pub ka: Vec3,
    pub p: f32,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    // position in world view (w = 1.0)
    pub pos: Vec4,

    // color of vertex, color = [0,1]x[0,1]x[0,1]
    pub color: Vec3,

    // true if apply shading to this vertex
    pub no_shade: bool,
}

impl Vertex {
    pub fn new(pos: Vec4, color: Vec3, no_shade: bool) -> Self {
        assert!(0.0 <= color.x && color.x <= 1.0);
        assert!(0.0 <= color.y && color.y <= 1.0);
        assert!(0.0 <= color.z && color.z <= 1.0);
        assert!(1.0 - f32::EPSILON <= pos.z && pos.z <= 1.0 + f32::EPSILON);

        Self {
            pos,
            color,
            no_shade,
        }
    }
}

/// Represents projected vertex, used for perspective correct interpolation
#[derive(Clone, Copy)]
pub struct RasterVertex {
    pub pos: Vec3,
    pub color: Vec3,
    pub inv_w: f32,
}

impl RasterVertex {
    pub fn new(pos: Vec4, color: Vec3) -> Self {
        Self {
            pos: pos.xyz() / pos.w,
            color,
            inv_w: 1.0 / pos.w,
        }
    }

    pub fn from_world_view(p: Vertex, m_cam: Mat4) -> Self {
        Self::new(m_cam * p.pos, p.color)
    }

    pub fn is_back_facing(a: Self, b: Self, c: Self) -> bool {
        let pa: Vec2 = a.pos.xy();
        let pb: Vec2 = b.pos.xy();
        let pc: Vec2 = c.pos.xy();

        // lte because higher z => further from screen
        (pb - pa).perp_dot(pc - pa) <= 0.0
    }

    pub fn is_inside(a: Self, b: Self, c: Self, p: Vec2) -> bool {
        let (pa, pb, pc): (Vec2, Vec2, Vec2) = (a.pos.xy(), b.pos.xy(), c.pos.xy());
        let (ab, bc, ca): (Vec2, Vec2, Vec2) = (pb - pa, pc - pb, pa - pc);
        let (ap, bp, cp): (Vec2, Vec2, Vec2) = (p - pa, p - pb, p - pc);

        let (apb, bpc, cpa): (f32, f32, f32) = (ap.perp_dot(ab), bp.perp_dot(bc), cp.perp_dot(ca));
        (apb >= 0.0 && bpc >= 0.0 && cpa >= 0.0) || (apb <= 0.0 && bpc <= 0.0 && cpa <= 0.0)
    }

    pub fn barycentric_coordinate(
        a: RasterVertex,
        b: RasterVertex,
        c: RasterVertex,
        p: Vec2,
    ) -> (f32, f32, f32) {
        let pa: Vec2 = a.pos.xy();
        let pb: Vec2 = b.pos.xy();
        let pc: Vec2 = c.pos.xy();

        let area: f32 = (pb - pa).perp_dot(pc - pa);

        (
            (pb - pa).perp_dot(p - pa) / area,
            (pc - pb).perp_dot(p - pb) / area,
            (pa - pc).perp_dot(p - pc) / area,
        )
    }

    pub fn interpolate_inv_w(
        a: RasterVertex,
        b: RasterVertex,
        c: RasterVertex,
        barycentric_coordinate: (f32, f32, f32),
    ) -> f32 {
        let (alpha, beta, gamma): (f32, f32, f32) = barycentric_coordinate;

        alpha * a.inv_w + beta * b.inv_w + gamma * c.inv_w
    }

    fn interpolate<T>(
        triangle: (RasterVertex, RasterVertex, RasterVertex),
        a_val: T,
        b_val: T,
        c_val: T,
        barycentric_coordinate: (f32, f32, f32),
        inv_w: f32,
    ) -> T
    where
        T: Clone + Copy + Add<Output = T> + Mul<f32, Output = T> + Div<f32, Output = T>,
    {
        let (a, b, c): (RasterVertex, RasterVertex, RasterVertex) = triangle;
        let (alpha, beta, gamma): (f32, f32, f32) = barycentric_coordinate;

        (a_val * a.inv_w * alpha + b_val * b.inv_w * beta + c_val * c.inv_w * gamma) / inv_w
    }

    pub fn interpolate_z(
        triangle: (RasterVertex, RasterVertex, RasterVertex),
        barycentric_coordinate: (f32, f32, f32),
        inv_w: f32,
    ) -> f32 {
        let (a, b, c): (RasterVertex, RasterVertex, RasterVertex) = triangle;
        Self::interpolate::<f32>(
            triangle,
            a.pos.z,
            b.pos.z,
            c.pos.z,
            barycentric_coordinate,
            inv_w,
        )
    }

    pub fn interpolate_color(
        triangle: (RasterVertex, RasterVertex, RasterVertex),
        barycentric_coordinate: (f32, f32, f32),
        inv_w: f32,
    ) -> Vec3 {
        let (a, b, c): (RasterVertex, RasterVertex, RasterVertex) = triangle;
        Self::interpolate::<Vec3>(
            triangle,
            a.color,
            b.color,
            c.color,
            barycentric_coordinate,
            inv_w,
        )
    }
}
