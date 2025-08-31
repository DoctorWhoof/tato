use super::*;

// Right panel
impl Dashboard {
    pub fn process_text_panel<const LEN: usize>(
        &mut self,
        layout: &mut Frame<i16>,
        frame_arena: &mut Arena<LEN>,
        backend: &impl Backend,
        tato: &Tato,
    ) {
        if self.debug_text.capacity() == 0 {
            return;
        }

        // Add debug info
        self.str("----------- Engine info -----------");

        let iter_time = backend.get_pixel_iter_elapsed_time();
        self.display("Pixel iter time: {:.1} ms", &[iter_time * 1000.0], "");

        let arena_cap = frame_arena.capacity();
        self.display(
            "Frame mem.: {:.1} / {:.1}",
            &[self.last_frame_arena_use as f32 / 1024.0, arena_cap as f32 / 1024.0],
            " Kb",
        );

        let fixed_arena_cap = self.fixed_arena.capacity();
        self.display(
            "Dash mem. (fixed): {:.1} / {:.1}",
            &[self.fixed_arena.used() as f32 / 1024.0, fixed_arena_cap as f32 / 1024.0],
            " Kb",
        );

        self.display(
            "Dash mem. (debug): {:.1} / {:.1}",
            &[self.debug_arena.used() as f32 / 1024.0, self.debug_arena.capacity() as f32 / 1024.0],
            " Kb",
        );

        self.display(
            "Asset mem.: {:.1} / {:.1}",
            &[
                tato.assets.arena.used() as f32 / 1024.0,
                tato.assets.arena.capacity() as f32 / 1024.0,
            ],
            " Kb",
        );

        self.display("fps: {:.1}", &[1.0 / tato.elapsed_time()], "");

        self.display("elapsed: {:.1}", &[tato.elapsed_time() * 1000.0], "");

        // Push Draw Ops to shared frame arena
        layout.push_edge(Edge::Left, PANEL_WIDTH, |panel| {
            panel.set_margin(5);
            panel.set_gap(0);
            let op =
                frame_arena.alloc(DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY }).unwrap();
            self.ops.push(frame_arena, op).unwrap();
            let items = self.debug_text.as_slice(&self.debug_arena).unwrap();
            for debug_text in items {
                // Adjust layout
                let mut rect = Rect::default();
                let mut line_height = 0.0;
                panel.push_edge(Edge::Top, self.font_size as i16, |text_frame| {
                    rect = text_frame.rect();
                    line_height = self.font_size * text_frame.get_scale();
                });
                // Text must be re-generated in new arena
                let text = Text::from_bytes(
                    frame_arena, //
                    debug_text.as_slice(&self.debug_arena).unwrap(),
                )
                .unwrap();
                // Create Op from new text object
                let op = {
                    DrawOp::Text {
                        text,
                        x: rect.x,
                        y: rect.y,
                        size: line_height,
                        color: RGBA32::WHITE,
                    }
                };
                let handle = frame_arena.alloc(op).unwrap();
                self.ops.push(frame_arena, handle).unwrap();
            }
        });
    }
}
