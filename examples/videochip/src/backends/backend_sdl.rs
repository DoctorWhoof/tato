use mini_sdl::*;
use tato::backend::*;
use tato::video::*;

pub struct Backend {
    app: App,
}

impl BackendVideo for Backend {
    fn new_window(vid: &VideoChip) -> Self {
        let mut app = App::new(
            "Test",
            vid.width(),
            vid.height(),
            Timing::Vsync,
            Scaling::Integer,
            // None,
        )
        .unwrap();

        app.init_pixel_buffer().unwrap();
        app.init_render_target().unwrap();

        app.print_fps_interval = Some(1.0 / 60.0);
        Self { app }
    }

    fn frame_start(&mut self, _vid: &VideoChip) {
        self.app.frame_start().unwrap();
    }

    fn frame_update(&mut self, vid: &VideoChip) {
        let width = self.app.width() as usize;
        self.app
            .pixel_buffer_update(|pixel, _pitch| {
                for (color, coords) in vid.iter_pixels() {
                    let i = ((coords.y as usize * width) + coords.x as usize) * 3;
                    pixel[i] = color.r;
                    pixel[i + 1] = color.g;
                    pixel[i + 2] = color.b;
                }
            })
            .unwrap();
    }

    fn frame_finish(&mut self, _vid: &VideoChip) {
        // self.app.calculate_update_time();
        self.app.pixel_buffer_present().unwrap();
        self.app.frame_finish().unwrap();
    }

    fn elapsed(&self) -> f64 {
        self.app.elapsed_time()
    }

    fn time(&self) -> f64 {
        self.app.time().as_secs_f64()
    }

    fn gamepad(&self) -> AnaloguePad {
        self.app.pad
    }

    fn quit_requested(&self) -> bool {
        self.app.quit_requested
    }
}
