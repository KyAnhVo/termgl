// the enums
pub mod options;
pub use options::{LightSourceShadingMode, ShadingMode};

// the typical shapes
mod mesh;
mod uv_map;
mod vertex;
pub use mesh::{Mesh, VertexIndices};
pub use uv_map::{HeightMap, NormalMap, UVMap};
pub use vertex::{Material, Vertex};

// the spaces
mod projection;
pub use projection::Camera;

// the lighting
mod point_light_source;
mod shader;
pub use point_light_source::PointLightSource;
pub use shader::Shader;

// to the screen
mod printer;
mod rasterizer;
pub use printer::{Printer, PrinterType};
pub use rasterizer::Rasterizer;

// the pipeline
mod pipeline3d;
pub use pipeline3d::Pipeline3D;
