use sdl2::{pixels::Color, render::Canvas, video::Window, Sdl};

pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new(sdl_context: &Sdl) -> Display {
        let canvas = Display::init_window(sdl_context);

        Display { canvas }
    }

    fn init_window(sdl_context: &Sdl) -> Canvas<Window> {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("CHIP 8 Emulator", 800, 600)
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
}
