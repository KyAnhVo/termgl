// the enums
mod options;
pub use options::ShadingMode;

// the typical shapes
mod mesh;
mod uv_map;
mod vertex;
pub use mesh::{Mesh, VertexIndices};
pub use uv_map::{HeightMap, NormalMap, UVMap};
pub use vertex::{Material, Vertex};

// the spaces
mod camera;
pub use camera::Camera;

// the lighting
mod point_light_source;
mod shader;
pub use point_light_source::PointLightSource;
pub use shader::Shader;

// to the screen
mod printer;
mod rasterizer;
pub use printer::PrinterType;
pub use rasterizer::Background;

// the pipeline
mod pipeline3d;
pub use pipeline3d::Pipeline3D;
