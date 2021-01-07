use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window, Sdl};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

pub struct Display {
    canvas: Canvas<Window>,
    scale: u32,
}

impl Display {
    pub fn new(sdl_context: &Sdl, scale: u32) -> Display {
        let canvas = Display::init_window(sdl_context, scale);

        Display { canvas, scale }
    }

    fn init_window(sdl_context: &Sdl, scale: u32) -> Canvas<Window> {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("CHIP 8 Emulator", WIDTH * scale, HEIGHT * scale)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        canvas
    }

    pub fn draw(&mut self, vram: &[[u8; 64]; 32]) {
        for (line_idx, line) in vram.iter().enumerate() {
            for (col_idx, pixel) in line.iter().enumerate() {
                let colour = get_pixel_colour(*pixel);

                let x = col_idx as i32 * self.scale as i32;
                let y = line_idx as i32 * self.scale as i32;

                self.canvas.set_draw_color(colour);
                let _ = self
                    .canvas
                    .fill_rect(Rect::new(x, y, self.scale, self.scale));
            }
        }

        self.canvas.present();
    }
}

fn get_pixel_colour(pixel_value: u8) -> Color {
    match pixel_value {
        0 => Color::BLACK,
        _ => Color::GREEN,
    }
}
