use super::*;

const TEMP_ARENA_LEN: usize = 16384;

// Right panel
impl<const LEN: usize> Dashboard<LEN> {
    pub fn process_text_panel(&mut self, layout: &mut Frame<i16>, tato: &Tato) {
        // Internal temp memory
        let mut temp = Arena::<TEMP_ARENA_LEN>::new();

        // Add debug info
        {
            {
                let arena_cap = self.temp_arena.capacity();
                let frame_text = Text::format_display(
                    &mut self.temp_arena,
                    "Dash mem. (temp): {:.1} / {:.1}",
                    &[self.last_frame_arena_use as f32 / 1024.0, arena_cap as f32 / 1024.0],
                    " Kb",
                );
                self.additional_text.push(&mut self.temp_arena, frame_text.unwrap()).unwrap();
            }

            {
                let fixed_arena_cap = self.fixed_arena.capacity();
                let frame_text = Text::format_display(
                    &mut self.temp_arena,
                    "Dash mem. (fixed): {:.1} / {:.1}",
                    &[self.fixed_arena.used() as f32 / 1024.0, fixed_arena_cap as f32 / 1024.0],
                    " Kb",
                );
                self.additional_text.push(&mut self.temp_arena, frame_text.unwrap()).unwrap();
            }

            let debug_text = Text::format_display(
                &mut self.temp_arena,
                "Tato Debug mem.: {:.1} / {:.1}",
                &[
                    tato.debug_arena.used() as f32 / 1024.0,
                    tato.debug_arena.capacity() as f32 / 1024.0,
                ],
                " Kb",
            );
            self.additional_text.push(&mut self.temp_arena, debug_text.unwrap()).unwrap();

            let asset_text = Text::format_display(
                &mut self.temp_arena,
                "Asset mem.: {:.1} / {:.1}",
                &[
                    tato.assets.arena.used() as f32 / 1024.0,
                    tato.assets.arena.capacity() as f32 / 1024.0,
                ],
                " Kb",
            );
            self.additional_text.push(&mut self.temp_arena, asset_text.unwrap()).unwrap();

            let fps_text = Text::format_display(
                &mut self.temp_arena,
                "fps: {:.1}",
                &[1.0 / tato.elapsed_time()],
                "",
            );
            self.additional_text.push(&mut self.temp_arena, fps_text.unwrap()).unwrap();

            let elapsed_text = Text::format_display(
                &mut self.temp_arena,
                "elapsed: {:.1}",
                &[tato.elapsed_time() * 1000.0],
                "",
            );
            self.additional_text.push(&mut self.temp_arena, elapsed_text.unwrap()).unwrap();

            let separator = Text::from_str(&mut self.temp_arena, "------------------------");
            self.additional_text.push(&mut self.temp_arena, separator.unwrap()).unwrap();

            for text in tato.iter_dash_text() {
                self.push_text(text);
            }
        }

        // Draw panel
        let mut temp_buffer = Buffer::<DrawOp>::new(&mut temp, 200).unwrap();
        layout.push_edge(Edge::Left, PANEL_WIDTH, |panel| {
            panel.set_margin(5);
            panel.set_gap(0);
            let op = self
                .temp_arena
                .alloc(DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY })
                .unwrap();
            self.ops.push(&mut self.temp_arena, op).unwrap();
            let items = self.additional_text.items(&self.temp_arena).unwrap();
            for text in items {
                let op = self.get_text_op(text.clone(), panel);
                temp_buffer.push(&mut temp, op).unwrap();
            }
        });

        for op in temp_buffer.items(&temp).unwrap() {
            let handle = self.temp_arena.alloc(op.clone()).unwrap();
            self.ops.push(&mut self.temp_arena, handle).unwrap()
        }
        temp.clear();
    }
}
