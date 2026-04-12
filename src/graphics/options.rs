/// Shading modes (no flat though)
#[derive(Clone, Copy, PartialEq)]
pub enum ShadingMode {
    /// Calculate vertex color, interpolate color into pixel
    Gouraud,
    /// interpolate normal into pixel, calculate pixel color
    Phong,
}

/// shading modes for light source
#[derive(Clone, Copy)]
pub enum LightSourceShadingMode {
    /// Lambertian emitter/scatterer has uniform look from all sides
    Lambertian,
    /// use n.dot(v) for surface normal and view normal, sometimes preferred
    LambertianCosineLaw,
}
