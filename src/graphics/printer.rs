use glam::Vec3;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PrinterType {
    Ascii,
    Color,
}

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
        Self {
            printer_type,
            width,
            height: height / 2,
            buff: vec![],
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height / 2;
    }

    pub fn print(&mut self, color: &[Vec3]) {
        self.buff.clear();
        self.buff.extend_from_slice(Printer::START_SEQUENCE);

        let mut curr_fg_color: Vec3 = Vec3::ZERO; // background color
        let mut curr_bg_color: Vec3 = Vec3::ZERO; // foreground color
        if self.printer_type == PrinterType::Color {
            self.buff.extend_from_slice(b"\x1b[38;2;0;0;0m");
            self.buff.extend_from_slice(b"\x1b[48;2;0;0;0m");
        };

        for i in 0..self.height {
            if cfg!(windows) {
                self.buff.push(b'\r');
            }
            self.buff.push(b'\n');
            let i_frame: usize = i * 2;
            for j in 0..self.width {
                let first_color: Vec3 = color[i_frame * self.width + j];
                let second_color: Vec3 = color[(i_frame + 1) * self.width + j];

                match &self.printer_type {
                    PrinterType::Ascii => {
                        // calculate avr color by add then div 2 will overflow,
                        // so we do this. Might induce error, but it's at most 2,
                        // so we dont really care.
                        let avr_color: Vec3 = (first_color + second_color) / 2.0;
                        let luminance: f32 = avr_color.element_sum() / 3.0;
                        let ramp_ind: usize = (luminance * (Self::RAMP.len() - 1) as f32) as usize;
                        let char_to_print: u8 = Self::RAMP[ramp_ind];
                        self.buff.push(char_to_print);
                    }
                    PrinterType::Color => {
                        if curr_fg_color != first_color {
                            curr_fg_color = first_color;
                            let color_code = format!(
                                "\x1b[38;2;{};{};{}m",
                                (curr_fg_color.x * 255.0) as u32,
                                (curr_fg_color.y * 255.0) as u32,
                                (curr_fg_color.z * 255.0) as u32
                            );
                            self.buff.extend_from_slice(color_code.as_bytes());
                        }
                        if curr_bg_color != second_color {
                            curr_bg_color = second_color;
                            let color_code = format!(
                                "\x1b[48;2;{};{};{}m",
                                (curr_bg_color.x * 255.0) as u32,
                                (curr_bg_color.y * 255.0) as u32,
                                (curr_bg_color.z * 255.0) as u32
                            );
                            self.buff.extend_from_slice(color_code.as_bytes());
                        }
                        self.buff.extend_from_slice("▀".as_bytes());
                    }
                };
            }
        }
    }
}
