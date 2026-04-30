use std::ops::{Add, Div, Mul};

use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

#[derive(Clone, Copy)]
pub struct Material {
    pub specular_constant: Vec3,
    pub ambient_constant: f32,
    pub specular_exponent: f32,
}

impl Material {
    pub fn new(specular_constant: Vec3, ambient_constant: f32, specular_exponent: f32) -> Self {
        Self {
            specular_constant,
            ambient_constant,
            specular_exponent,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vertex {
    // position in world view (w = 1.0)
    pub pos: Vec4,
}

impl Vertex {
    pub fn from_vec3(pos: Vec3) -> Self {
        Self {
            pos: pos.extend(1.0),
        }
    }
    pub fn from_vec4(pos: Vec4) -> Self {
        assert!(pos.w == 1.0, "default position w must be 0");
        Self { pos }
    }
}

/// Represents projected vertex, used for perspective correct interpolation
#[derive(Clone, Copy)]
pub struct RasterVertex {
    pub pos: Vec3,
    pub inv_w: f32,
}

impl RasterVertex {
    pub fn new(pos: Vec4) -> Self {
        Self {
            pos: pos.xyz() / pos.w,
            inv_w: 1.0 / pos.w,
        }
    }

    pub fn from_world_view(p: Vertex, m_cam: Mat4) -> Self {
        Self::new(m_cam * p.pos)
    }

    pub fn is_back_facing(a: Self, b: Self, c: Self) -> bool {
        let pa: Vec2 = a.pos.xy();
        let pb: Vec2 = b.pos.xy();
        let pc: Vec2 = c.pos.xy();

        // gte because higher z => further from screen
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
            (pb - p).perp_dot(pc - p) / area,
            (pc - p).perp_dot(pa - p) / area,
            (pa - p).perp_dot(pb - p) / area,
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
        color: (Vec3, Vec3, Vec3),
        barycentric_coordinate: (f32, f32, f32),
        inv_w: f32,
    ) -> Vec3 {
        Self::interpolate::<Vec3>(
            triangle,
            color.0,
            color.1,
            color.2,
            barycentric_coordinate,
            inv_w,
        )
    }

    pub fn interpolate_normals(
        triangle: (RasterVertex, RasterVertex, RasterVertex),
        normals: (Vec4, Vec4, Vec4),
        barycentric_coordinate: (f32, f32, f32),
        inv_w: f32,
    ) -> Vec4 {
        Self::interpolate::<Vec4>(
            triangle,
            normals.0,
            normals.1,
            normals.2,
            barycentric_coordinate,
            inv_w,
        )
    }

    pub fn interpolate_position(
        triangle: (RasterVertex, RasterVertex, RasterVertex),
        world_space_triangle: (Vec4, Vec4, Vec4),
        barycentric_coordinate: (f32, f32, f32),
        inv_w: f32,
    ) -> Vec4 {
        Self::interpolate::<Vec4>(
            triangle,
            world_space_triangle.0,
            world_space_triangle.1,
            world_space_triangle.2,
            barycentric_coordinate,
            inv_w,
        )
    }

    pub fn interpolate_uv(
        triangle: (RasterVertex, RasterVertex, RasterVertex),
        uv_triangle: (Vec2, Vec2, Vec2),
        barycentric_coordinate: (f32, f32, f32),
        inv_w: f32,
    ) -> Vec2 {
        Self::interpolate::<Vec2>(
            triangle,
            uv_triangle.0,
            uv_triangle.1,
            uv_triangle.2,
            barycentric_coordinate,
            inv_w,
        )
    }
}
