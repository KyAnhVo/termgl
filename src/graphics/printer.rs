#[derive(PartialEq, Eq)]
pub enum PrinterType {
    Ascii,
    Color,
}

use crate::graphics::triangle::{self, Color};

pub struct Printer {
    pub printer_type: PrinterType,
    pub width: usize,
    pub height: usize,
    pub buff: Vec<u8>,
}

impl Printer {
    const START_SEQUENCE: &[u8] = b"\x1b[H\x1b[?25l";
    const RAMP: &[u8] = b" .:-=+*#%@";

    pub fn new(printer_type: PrinterType, width: usize, height: usize) -> Self {
        let v_height = height / 2; // Terminal rows for half-block rendering (estimatedly)
        Self {
            printer_type,
            width,
            height: v_height,
            buff: vec![],
        }
    }

    pub fn print(&mut self, color: &mut [Color]) {
        self.buff.clear();
        self.buff.extend_from_slice(Printer::START_SEQUENCE);

        let mut curr_color: Color = Color::BLACK; // background color
        if self.printer_type == PrinterType::Color {
            self.buff.extend_from_slice(b"\x1b[38;2;0;0;0m");
        };

        for i in 0..self.height {
            if cfg!(windows) {
                self.buff.push(b'\r');
            }
            self.buff.push(b'\n');
            let i_frame: usize = i * 2;
            for j in 0..self.width {
                let first_color: Color = color[i_frame * self.width + j];
                let second_color: Color = color[(i_frame + 1) * self.width + j];
                // calculate avr color by add then div 2 will overflow,
                // so we do this. Might induce error, but it's at most 2,
                // so we dont really care.
                let avr_color: Color = Color::new(
                    ((first_color.r as u32 + second_color.r as u32) / 2) as u8, 
                    ((first_color.g as u32 + second_color.g as u32) / 2) as u8, 
                    ((first_color.b as u32 + second_color.b as u32) / 2) as u8, 
                );
                
                match self.printer_type {
                    PrinterType::Ascii => {
                        let luminance: f32  = (avr_color.r as f32 * 0.2126 +
                                               avr_color.g as f32 * 0.7152 +
                                               avr_color.b as f32 * 0.0722) / 255.0;
                        let ramp_ind: usize = (luminance * (Self::RAMP.len() - 1) as f32) as usize;
                        let char_to_print: u8 = Self::RAMP[ramp_ind];
                        self.buff.push(char_to_print);
                    }
                    PrinterType::Color => {
                        if curr_color != avr_color {
                            curr_color = avr_color;
                            let color_code = format!("\x1b[38;2;{};{};{}m", curr_color.r, curr_color.g, curr_color.b);
                            self.buff.extend_from_slice(color_code.as_bytes());
                        }
                        self.buff.extend_from_slice("█".as_bytes());
                    }
                };
            }

        }
    }

}
