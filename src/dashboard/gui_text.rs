use super::*;

// Right panel
impl Dashboard {
    pub(super) fn process_text_panel<A>(
        &mut self,
        layout: &mut Frame<i16>,
        frame_arena: &mut A,
        backend: &impl Backend,
        tato: &Tato,
    ) where
        A: ArenaOps<u32, ()>,
    {
        if self.debug_text.capacity() == 0 {
            return;
        }

        // Add debug info
        self.str(frame_arena, "");
        self.str(frame_arena, "----------- Engine info -----------");

        let draw_time = backend.get_drawing_elapsed_time();
        let iter_time = backend.get_pixel_iter_elapsed_time();

        self.display_txt(
            frame_arena,
            "fps: {:.1} / {:.0}",
            &[1.0 / tato.elapsed_time(), (1.0 / (iter_time + draw_time))],
            "",
        );
        self.display_txt(frame_arena, "elapsed: {:.1}", &[tato.elapsed_time() * 1000.0], "");
        self.display_txt(frame_arena, "Backend pixel iter time: {:.1} ms", &[iter_time * 1000.0], "");
        self.display_txt(frame_arena, "Backend draw time: {:.1} ms", &[draw_time * 1000.0], "");

        let arena_cap = frame_arena.capacity();
        self.display_txt(
            frame_arena,
            "Shared Frame Mem.: {:.1} / {:.1}",
            &[self.last_frame_arena_use as f32 / 1024.0, arena_cap as f32 / 1024.0],
            " Kb",
        );

        let fixed_arena_cap = self.fixed_arena.capacity();
        self.display_txt(
            frame_arena,
            "Dash Mem. (fixed): {:.1} / {:.1}",
            &[self.fixed_arena.used() as f32 / 1024.0, fixed_arena_cap as f32 / 1024.0],
            " Kb",
        );

        self.display_txt(
            frame_arena,
            "Dash DrawOps: {} / {}",
            &[self.last_frame_draw_op_count as u32, self.ops.capacity()],
            "",
        );

        // Push Draw Ops to shared frame arena
        layout.push_edge(Edge::Left, PANEL_WIDTH, |panel| {
            panel.set_margin(5);
            panel.set_gap(0);
            let op =
                frame_arena.alloc(DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY }).unwrap();
            self.ops.push(frame_arena, op).unwrap();

            // Use index-based iteration to avoid holding the arena borrow
            let debug_text_len = self.debug_text.len();
            for i in 0..debug_text_len {
                // Get text at index i - borrow is released immediately after copying
                let Some(debug_text) = self.debug_text.get(frame_arena, i) else {
                    break;
                };
                // Adjust layout
                let mut rect = Rect::default();
                let mut line_height = 0.0;
                panel.push_edge(Edge::Top, self.font_size as i16, |text_frame| {
                    rect = text_frame.rect();
                    line_height = self.font_size * text_frame.get_scale();
                });
                // Create Op from text object
                let op = DrawOp::Text {
                    text: debug_text,
                    x: rect.x,
                    y: rect.y,
                    size: line_height,
                    color: RGBA32::WHITE,
                };
                let handle = frame_arena.alloc(op).unwrap();
                self.ops.push(frame_arena, handle).unwrap();
            }
        });
    }
}
