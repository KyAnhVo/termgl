mod graphics;
mod physics;

use std::io::{stdout, Write};
use crate::graphics::vertex::{RasterVertex};
use crate::graphics::printer::{Printer, PrinterType};
use crate::graphics::rasterizer::{Rasterizer};

use crossterm::terminal;
use glam::{Vec3, Vec4};

use std::{thread, time};


fn main() {
    let (width_u16, height_u16) = terminal::size().unwrap();
    let (width, height) = (width_u16 as usize, height_u16 as usize * 2);
    test_rast(width, height);
}

fn test_rast(w: usize, h: usize) {
    let v1: RasterVertex = RasterVertex::new(Vec4::new(-1.0, 0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
    let v2: RasterVertex = RasterVertex::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
    let v3: RasterVertex = RasterVertex::new(Vec4::new(0.0, -1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
    
    let mut rast: Rasterizer = Rasterizer::new(w, h, Vec3::new(0.0, 0.1, 0.3));
    rast.clear();
    rast.render_triangle(v1, v3, v2);
    thread::sleep(time::Duration::from_secs(2));

    let mut printer: Printer = Printer::new(PrinterType::Color, w, h);
    printer.print(&mut rast.frame_buff);
    stdout().write_all(&printer.buff).unwrap();
}
