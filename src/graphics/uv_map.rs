use glam::{Mat3, Vec2, Vec3, Vec3Swizzles};
use image::{ImageReader, RgbImage};

/// A height map that uses a UV map to sample height values.
/// This implements the parallaxed UV interpolation algorithm,
/// so use this before using the other uv maps.
pub struct HeightMap {
    map: UVMap,
    pub height_scale: f32,
}

impl HeightMap {
    pub fn new(path: &str, height_scale: f32) -> Self {
        Self {
            map: UVMap::new(path),
            height_scale,
        }
    }

    /// Interpolates the parallaxed UV coordinates for the given UV and view direction.
    pub fn interpolate_parallaxed_uv(&self, uv: Vec2, view_dir: Vec3) -> Vec2 {
        let color: Vec3 = self.map.interpolate(uv);
        let height: f32 = color.x;
        uv - view_dir.xy() / view_dir.z * (height * self.height_scale)
    }
}

/// A normal map that uses a UV map to sample normal values.
pub struct NormalMap {
    /// the uv map used to sample normals
    map: UVMap,
}

impl NormalMap {
    pub fn new(path: &str) -> Self {
        Self {
            map: UVMap::new(path),
        }
    }

    /// Interpolates the normal for the given UV and triangle vertices.
    pub fn interpolate(
        &self,
        uv: Vec2,
        triangle_vertices: (Vec3, Vec3, Vec3),
        triangle_uvs: (Vec2, Vec2, Vec2),
    ) -> Vec3 {
        let color: Vec3 = self.map.interpolate(uv);
        let normal: Vec3 = color * 2.0 - Vec3::ONE;

        // implement the TBN matrix
        // we use the following formula:
        // [T, B] = [E1, E2] * [d(UV)1, d(UV)2].inv
        // where [E1, E2] is the triangle's edge vectors
        // and [d(UV)1, d(UV)2] is the triangle's UV edge vectors
        // and N is the triangle's normal vector
        // hence we can compute T, N and then let B = N x T

        let e1: Vec3 = triangle_vertices.1 - triangle_vertices.0;
        let e2: Vec3 = triangle_vertices.2 - triangle_vertices.0;
        let duv1: Vec2 = triangle_uvs.1 - triangle_uvs.0;
        let duv2: Vec2 = triangle_uvs.2 - triangle_uvs.0;
        let f: f32 = 1.0 / (duv1.x * duv2.y - duv1.y * duv2.x);

        let tx: f32 = f * (duv2.y * e1.x - duv1.y * e2.x);
        let ty: f32 = f * (duv2.y * e1.y - duv1.y * e2.y);
        let tz: f32 = f * (duv2.y * e1.z - duv1.y * e2.z);

        let t_pre_gram_schmidt: Vec3 = Vec3::new(tx, ty, tz);
        let n: Vec3 = (e1.cross(e2)).normalize();
        let t: Vec3 = (t_pre_gram_schmidt - n * n.dot(t_pre_gram_schmidt)).normalize();
        let b: Vec3 = n.cross(t).normalize();
        let tbn: Mat3 = Mat3::from_cols(t, b, n);

        tbn * normal
    }
}

/// A UV map that uses bilinear interpolation to sample colors.
/// By default, use this for the texture map.
pub struct UVMap {
    pub buff: RgbImage,
}

impl UVMap {
    pub fn new(path: &str) -> Self {
        // Note: unwrap since panic here is better than returning an error,
        // since a mesh that uses this UV map will likely:
        // - need the color that the map provides
        // - not have a fallback color to use if the map is not available
        Self {
            buff: ImageReader::open(path).unwrap().decode().unwrap().to_rgb8(),
        }
    }

    /// implement bilinear interpolation
    /// returns the interpolated color at the given UV coordinate
    /// in the rectangle [0, 1] x [0, 1] x [0, 1]
    pub fn interpolate(&self, uv: Vec2) -> Vec3 {
        // a lot of typecasting here, expect the compiler
        // to optimize away most of it even at optimization level 1
        let (udim_u, udim_v): (u32, u32) = self.buff.dimensions();
        let (dim_u, dim_v): (f32, f32) = (udim_u as f32, udim_v as f32);
        let (u, v): (f32, f32) = (uv.x * dim_u, uv.y * dim_v);
        let (u_low, v_low): (f32, f32) = (u.floor(), v.floor());
        let (u_high, v_high): (f32, f32) = (u_low + 1.0, v_low + 1.0);

        // tu for interpolating u pos, tv for interpolating v pos
        // t on low side, 1.0 - t on high side
        let (tu, tv): (f32, f32) = (u - u_low, v - v_low);

        // description is u first then v, so low_high_something
        // is equivalent to (u_low, v_high)
        let low_low_rgb = self.buff.get_pixel(u_low as u32, v_low as u32);
        let low_high_rgb = self.buff.get_pixel(u_low as u32, v_high as u32);
        let high_low_rgb = self.buff.get_pixel(u_high as u32, v_low as u32);
        let high_high_rgb = self.buff.get_pixel(u_high as u32, v_high as u32);

        let low_low: Vec3 = Vec3::new(
            low_low_rgb[0] as f32,
            low_low_rgb[1] as f32,
            low_low_rgb[2] as f32,
        ) / 255.0;
        let low_high: Vec3 = Vec3::new(
            low_high_rgb[0] as f32,
            low_high_rgb[1] as f32,
            low_high_rgb[2] as f32,
        ) / 255.0;
        let high_low: Vec3 = Vec3::new(
            high_low_rgb[0] as f32,
            high_low_rgb[1] as f32,
            high_low_rgb[2] as f32,
        ) / 255.0;
        let high_high: Vec3 = Vec3::new(
            high_high_rgb[0] as f32,
            high_high_rgb[1] as f32,
            high_high_rgb[2] as f32,
        ) / 255.0;

        // basic bilinear interpolation
        let low_lerp = low_low * tv + low_high * (1.0 - tv);
        let high_lerp = high_low * tv + high_high * (1.0 - tv);
        low_lerp * tu + high_lerp * (1.0 - tu)
    }
}
