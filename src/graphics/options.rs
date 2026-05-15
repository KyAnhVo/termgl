/// Shading modes (no flat though)
#[derive(Clone, Copy, PartialEq)]
pub enum ShadingMode {
    /// Calculate vertex color, interpolate color into pixel
    Gouraud,
    /// interpolate normal into pixel, calculate pixel color
    Phong,
}
