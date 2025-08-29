use super::*;

const TEMP_ARENA_LEN: usize = 16384;

// Right panel
impl Dashboard {
    pub fn process_text_panel<const LEN: usize>(
        &mut self,
        layout: &mut Frame<i16>,
        frame_arena: &mut Arena<LEN, u32>,
        backend: &impl Backend,
        tato: &Tato,
    ) {
        // Add debug info
        {
            {
                let iter_time = backend.get_pixel_iter_elapsed_time();
                let iter_text = Text::format_display(
                    frame_arena,
                    "Pixel iter time: {:.1} ms", //
                    &[iter_time * 1000.0],
                    "",
                );
                self.additional_text.push(frame_arena, iter_text.unwrap()).unwrap();
            }

            {
                let arena_cap = frame_arena.capacity();
                let frame_text = Text::format_display(
                    frame_arena,
                    "Frame mem.: {:.1} / {:.1}",
                    &[self.last_frame_arena_use as f32 / 1024.0, arena_cap as f32 / 1024.0],
                    " Kb",
                );
                self.additional_text.push(frame_arena, frame_text.unwrap()).unwrap();
            }

            {
                let fixed_arena_cap = self.fixed_arena.capacity();
                let frame_text = Text::format_display(
                    frame_arena,
                    "Dash mem. (fixed): {:.1} / {:.1}",
                    &[self.fixed_arena.used() as f32 / 1024.0, fixed_arena_cap as f32 / 1024.0],
                    " Kb",
                );
                self.additional_text.push(frame_arena, frame_text.unwrap()).unwrap();
            }

            let debug_text = Text::format_display(
                frame_arena,
                "Tato Debug mem.: {:.1} / {:.1}",
                &[
                    tato.debug_arena.used() as f32 / 1024.0,
                    tato.debug_arena.capacity() as f32 / 1024.0,
                ],
                " Kb",
            );
            self.additional_text.push(frame_arena, debug_text.unwrap()).unwrap();

            let asset_text = Text::format_display(
                frame_arena,
                "Asset mem.: {:.1} / {:.1}",
                &[
                    tato.assets.arena.used() as f32 / 1024.0,
                    tato.assets.arena.capacity() as f32 / 1024.0,
                ],
                " Kb",
            );
            self.additional_text.push(frame_arena, asset_text.unwrap()).unwrap();

            let fps_text =
                Text::format_display(frame_arena, "fps: {:.1}", &[1.0 / tato.elapsed_time()], "");
            self.additional_text.push(frame_arena, fps_text.unwrap()).unwrap();

            let elapsed_text = Text::format_display(
                frame_arena,
                "elapsed: {:.1}",
                &[tato.elapsed_time() * 1000.0],
                "",
            );
            self.additional_text.push(frame_arena, elapsed_text.unwrap()).unwrap();

            let separator = Text::from_str(frame_arena, "------------------------");
            self.additional_text.push(frame_arena, separator.unwrap()).unwrap();

            for text in tato.iter_dash_text() {
                self.push_text(text, frame_arena);
            }
        }

        // Internal temp memory. Required to avoid borrow issues
        let mut temp = Arena::<TEMP_ARENA_LEN>::new();
        let mut temp_buffer = Buffer::<DrawOp>::new(&mut temp, 200).unwrap();
        // Draw panel
        layout.push_edge(Edge::Left, PANEL_WIDTH, |panel| {
            panel.set_margin(5);
            panel.set_gap(0);
            let op =
                frame_arena.alloc(DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY }).unwrap();
            self.ops.push(frame_arena, op).unwrap();
            let items = self.additional_text.items(&frame_arena).unwrap();
            for text in items {
                let op = self.get_text_op(text.clone(), panel);
                temp_buffer.push(&mut temp, op).unwrap();
            }
        });

        for op in temp_buffer.items(&temp).unwrap() {
            let handle = frame_arena.alloc(op.clone()).unwrap();
            self.ops.push(frame_arena, handle).unwrap()
        }
        temp.clear();
    }
}
