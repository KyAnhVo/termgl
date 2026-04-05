use crate::graphics::{
    mesh::Mesh,
    options::ShadingMode,
    printer::{Printer, PrinterType},
    projection::Camera,
    rasterizer::Rasterizer,
    shader::Shader,
    vertex::{Material, RasterVertex, Vertex},
};
use crossterm::terminal;
use glam::{Mat3, Mat4, Vec3, Vec4, Vec4Swizzles};
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
    /// meshes to be rendered, original
    pub shader: Shader,
}

impl Pipeline3D {
    pub fn new(default_background: Vec3, printer_type: PrinterType, camera: Camera) -> Self {
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
        }
    }

    /// Resize pipeline's buffers, for printer and rasterizer.
    pub fn resize(&mut self) {
        let (width_u16, height_u16) = terminal::size().unwrap();
        let (width, height) = (width_u16 as usize, height_u16 as usize * 2);
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.rasterizer.resize(width, height);
            self.printer.resize(width, height);
        }
    }

    /// Render the meshes into rasterizer's buffer
    pub fn render_mesh(&mut self, mesh: &mut Mesh, shade_mode: ShadingMode) {
        // finalize mesh normals and vertices (to world space)
        mesh.finalize_normals();

        // transform to ndc-space
        let m_obj_to_ndc: Mat4 = self.camera.m_perspective(0.1, 10000000.0)
            * self.camera.m_view()
            * mesh.m_to_world_space();
        mesh.projected_vao.clear();
        for v in &mesh.vao {
            mesh.projected_vao
                .push(RasterVertex::from_world_view(*v, m_obj_to_ndc));
        }

        // shade vertices for Gouraud shading
        if shade_mode == ShadingMode::Gouraud || shade_mode == ShadingMode::Flat {
            self.shader.shade_mesh_gouraud(mesh, self.camera);
            println!("enter gouraud");
        } else {
            println!("skip gouraud");
        }

        let is_phong: bool = (shade_mode == ShadingMode::Phong) && !mesh.no_shade;
        self.rasterizer
            .rasterize_mesh(mesh, &self.shader, self.camera, is_phong,);
    }

    /// Print the buffer into the screen
    pub fn print(&mut self) {
        self.printer.print(&self.rasterizer.frame_buff);
        stdout().flush().unwrap();
        stdout().write_all(&self.printer.buff).unwrap();
    }
}
