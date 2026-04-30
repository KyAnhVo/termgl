use crate::graphics::{
    mesh::Mesh,
    options::ShadingMode,
    printer::{Printer, PrinterType},
    projection::Camera,
    rasterizer::Rasterizer,
    shader::Shader,
    vertex::RasterVertex,
};
use crossterm::terminal;
use glam::{Mat4, Vec3};
use std::io::{Write, stdout};

pub struct Pipeline3D {
    /// screen width
    pub width: usize,
    /// screen height
    pub height: usize,
    /// Rasterizer: [-1, 1] x [-1, 1] -> buffer
    pub rasterizer: Rasterizer,
    /// Printer to print the buffer to the terminal
    pub printer: Printer,
    /// Assume 1 camera
    pub camera: Camera,
    /// shader to shade the meshes
    pub shader: Shader,
    /// shading mode for the meshes
    pub shading_mode: ShadingMode,
}

impl Pipeline3D {
    pub fn new(
        default_background: Vec3,
        printer_type: PrinterType,
        camera: Camera,
        shading_mode: ShadingMode,
    ) -> Self {
        let (width_u16, height_u16) = terminal::size().unwrap();
        let (width, height) = (width_u16 as usize, height_u16 as usize * 2);

        let rasterizer: Rasterizer = Rasterizer::new(width, height, default_background);
        let printer: Printer = Printer::new(printer_type, width, height);
        let shader: Shader = Shader::new();

        Self {
            width,
            height,
            rasterizer,
            printer,
            camera,
            shader,
            shading_mode,
        }
    }

    /// Resize pipeline's buffers, for printer and rasterizer.
    fn resize(&mut self) {
        let (width_u16, height_u16) = terminal::size().unwrap();
        let (width, height) = (width_u16 as usize, height_u16 as usize * 2);
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.camera.resize(width, height);
            self.rasterizer.resize(width, height);
            self.printer.resize(width, height);
        }
    }

    /// Call when frame starts
    pub fn start_frame(&mut self) {
        self.rasterizer.clear();
        self.resize();
    }

    /// Render the meshes into rasterizer's buffer
    pub fn render_mesh(&mut self, mesh: &mut Mesh) {
        let shading_mode: ShadingMode = self.shading_mode;

        // finalize mesh normals and vertices (to world space)
        mesh.finalize_mesh();

        // transform to ndc-space
        let m_obj_to_ndc: Mat4 = self.camera.m_perspective(0.1, 10000000.0)
            * self.camera.m_view()
            * mesh.m_to_world_space();
        mesh.raster_vertices.clear();
        for v in &mesh.vertices {
            mesh.raster_vertices
                .push(RasterVertex::from_world_view(*v, m_obj_to_ndc));
        }

        self.rasterizer
            .rasterize_mesh(mesh, &self.shader, &self.camera, shading_mode);
    }

    /// Print the buffer into the screen
    fn print(&mut self) {
        self.printer.print(&self.rasterizer.frame_buff);
        stdout().flush().unwrap();
        stdout().write_all(&self.printer.buff).unwrap();
    }

    /// Call when frame ends
    pub fn end_frame(&mut self) {
        self.print();
    }
}
