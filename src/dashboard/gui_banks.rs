use super::*;

// Right panel
impl Dashboard {
    pub(super) fn process_video_banks_panel<const LEN: usize>(
        &mut self,
        layout: &mut Frame<i16>,
        frame_arena: &mut Arena<LEN>,
        backend: &impl Backend,
        tato: &Tato,
    ) {
        layout.push_edge(Edge::Right, PANEL_WIDTH, |panel| {
            panel.set_margin(5);
            panel.set_gap(0);
            panel.set_scale(self.gui_scale);
            panel.fitting = Fitting::Clamp;

            let rect_handle =
                frame_arena.alloc(DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY });
            self.ops.push(frame_arena, rect_handle.unwrap()).unwrap();

            // Process each video memory bank
            for bank_index in 0..TILE_BANK_COUNT {
                // Draw each bank debug data
                self.process_bank(frame_arena, backend, panel, bank_index, tato);
                // Small separator
                panel.push_edge(Edge::Top, 5, |_separator| {});
            }
        });
    }

    fn update_tile_texture(
        &mut self,
        bank_index: usize,
        bank: &VideoBank<{ TILE_COUNT }>,
        tiles_per_row: u16,
    ) {
        // Early return for empty banks
        if bank.tile_count() == 0 {
            // self.tile_pixels[bank_index].clear();
            return;
        }

        // Calculate actual dimensions based on tile layout
        let tile_count = bank.tile_count() as u16;
        let num_rows = (tile_count + tiles_per_row - 1) / tiles_per_row; // Ceiling division

        let w = tiles_per_row as usize * TILE_SIZE as usize;
        let h = num_rows as usize * TILE_SIZE as usize;
        let expected_size = w * h * 4; // RGBA

        // TODO: This may run of of space... we're reallocating within the fixed_arena!
        // May need to reset entire fixed_arena if a single bank doesn't match.
        // Needs testing, I think I'm not running into a problem simply because the pixel count
        // always matches
        if expected_size != self.tile_pixels[bank_index].len() {
            // Allocate buffer with correct size
            self.tile_pixels[bank_index].resize(&mut self.fixed_arena, expected_size as u32);

            // Generate tile pixels
            let pixels = self.tile_pixels[bank_index].as_slice_mut(&mut self.fixed_arena).unwrap();

            for tile_index in 0..tile_count {
                let tile_x = tile_index % tiles_per_row;
                let tile_y = tile_index / tiles_per_row;

                for y in 0..TILE_SIZE as usize {
                    for x in 0..TILE_SIZE as usize {
                        // get color
                        let color_index =
                            bank.tiles[tile_index as usize].get_pixel(x as u8, y as u8);
                        let gray_value = color_index * 63; // Map 0-4 to 0-252
                        // get coordinates
                        let pixel_x = tile_x as usize * TILE_SIZE as usize + x;
                        let pixel_y = tile_y as usize * TILE_SIZE as usize + y;
                        let i = ((pixel_y * w as usize) + pixel_x) * 4;

                        // Seems safe for now, may need to insert a check for i < pixels.len()
                        // if I get out-of-bounds errors.
                        pixels[i] = gray_value; // R
                        pixels[i + 1] = gray_value; // G
                        pixels[i + 2] = gray_value; // B
                        pixels[i + 3] = 255; // A
                    }
                }
            }
        }
    }

    fn process_bank<const LEN: usize>(
        &mut self,
        frame_arena: &mut Arena<LEN>,
        backend: &impl Backend,
        panel: &mut Frame<i16>,
        bank_index: usize,
        tato: &Tato,
    ) {
        let tiles_per_row = ((TILE_COUNT as f64).sqrt().ceil()) as u16;
        let tile_size = panel.rect().w as f32 / tiles_per_row as f32;

        let gap = self.gui_scale as i16;
        let bank = &tato.banks[bank_index];

        let mouse = backend.get_mouse();

        // Bank label
        let h = self.font_size as i16;
        panel.push_edge(Edge::Top, h, |frame| {
            let rect = frame.rect();
            let text = Text::format_display(frame_arena, "bank: {}", &[bank_index], "").unwrap();
            let handle = frame_arena
                .alloc(DrawOp::Text {
                    text,
                    x: rect.x + gap,
                    y: rect.y,
                    size: self.font_size * self.gui_scale,
                    color: RGBA32::WHITE,
                })
                .unwrap();
            self.ops.push(frame_arena, handle).unwrap();
        });

        // Bank info
        panel.push_edge(Edge::Top, h, |frame| {
            let rect = frame.rect();
            let values = [bank.tile_count(), bank.color_count() as usize];
            let text = Text::format_display(
                frame_arena,
                "{} tiles, {} colors",
                &values,
                "",
            )
            .unwrap();

            let handle = frame_arena
                .alloc(DrawOp::Text {
                    text,
                    x: rect.x + gap,
                    y: rect.y,
                    size: self.font_size * 0.75 * self.gui_scale,
                    color: RGBA32::WHITE,
                })
                .unwrap();
            self.ops.push(frame_arena, handle).unwrap();
        });

        if bank.tile_count() == 0 && bank.color_count() == 0 {
            return;
        }

        // Color palette swatches
        panel.push_edge(Edge::Top, 8, |frame| {
            let rect = frame.rect();
            let rect_handle =
                frame_arena.alloc(DrawOp::Rect { rect, color: DARKEST_GRAY }).unwrap();
            self.ops.push(frame_arena, rect_handle).unwrap();

            let swatch_w = frame.divide_width(COLORS_PER_PALETTE as u32);
            for c in 0..COLORS_PER_PALETTE as usize {
                frame.push_edge(Edge::Left, swatch_w, |swatch| {
                    let rect = swatch.rect();
                    let color = bank.palette[c];
                    let rgba32 = RGBA32::from(color);

                    let handle = frame_arena.alloc(DrawOp::Rect { rect, color: rgba32 }).unwrap();
                    self.ops.push(frame_arena, handle).unwrap();

                    // Mouse hover detection
                    if rect.contains(mouse.x, mouse.y) {
                        self.mouse_over_text = Text::format_display(
                            frame_arena,
                            "Color {} = {}, {}, {}, {}",
                            &[c as u8, color.r(), color.g(), color.b(), color.a()],
                            "",
                        )
                        .unwrap();
                    }
                });
            }
        });

        // // Sub-palette swatches
        // {
        //     let columns = 6;
        //     let rows = (bank.sub_palette_count() as f32 / columns as f32).ceil() as u32;
        //     let frame_h = (rows as i16 * 4) + 4;

        //     panel.push_edge(Edge::Top, frame_h, |frame| {
        //         frame.set_margin(1);
        //         frame.set_gap(1);
        //         let column_w = frame.divide_width(columns);
        //         for column in 0..columns {
        //             frame.push_edge(Edge::Left, column_w, |frame_column| {
        //                 frame_column.set_gap(0);
        //                 frame_column.set_margin(1);

        //                 let rect = frame_column.rect();
        //                 let rect_handle =
        //                     frame_arena.alloc(DrawOp::Rect { rect, color: DARKEST_GRAY }).unwrap();
        //                 self.ops.push(frame_arena, rect_handle).unwrap();

        //                 let row_h = frame_column.divide_height(rows);
        //                 for row in 0..rows {
        //                     frame_column.push_edge(Edge::Top, row_h, |frame_row| {
        //                         // frame_row.set_gap(1);
        //                         frame_row.set_margin(1);
        //                         let subp_index = ((row * COLORS_PER_TILE as u32) + column) as usize;
        //                         let current_item = (row * columns) + column;

        //                         if current_item < bank.sub_palette_count() as u32
        //                             && subp_index < bank.sub_palettes.len()
        //                         {
        //                             let subp = &bank.sub_palettes[subp_index];
        //                             let swatch_w = frame_row.divide_width(COLORS_PER_TILE as u32);

        //                             for n in 0..COLORS_PER_TILE as usize {
        //                                 frame_row.push_edge(Edge::Left, swatch_w, |swatch| {
        //                                     let swatch_rect = swatch.rect();
        //                                     let color_index = subp[n].0 as usize;
        //                                     if color_index < bank.palette.len() {
        //                                         let sub_rect_handle = frame_arena
        //                                             .alloc(DrawOp::Rect {
        //                                                 rect: swatch_rect,
        //                                                 color: RGBA32::from(
        //                                                     bank.palette[color_index],
        //                                                 ),
        //                                             })
        //                                             .unwrap();
        //                                         self.ops
        //                                             .push(frame_arena, sub_rect_handle)
        //                                             .unwrap();
        //                                     }
        //                                 });
        //                             }

        //                             // Mouse hover detection
        //                             if frame_row.rect().contains(mouse.x as i16, mouse.y as i16) {
        //                                 let colors = [
        //                                     subp_index as u8,
        //                                     subp[0].0,
        //                                     subp[1].0,
        //                                     subp[2].0,
        //                                     subp[3].0,
        //                                 ];
        //                                 self.mouse_over_text = Text::format_dbg(
        //                                     frame_arena,
        //                                     "Sub Palette {} = [{},{},{},{}]",
        //                                     &colors,
        //                                     "",
        //                                 )
        //                                 .unwrap();
        //                             }
        //                         }
        //                     });
        //                 }
        //             });
        //         }
        //     });
        // }

        // Tile visualization
        self.update_tile_texture(bank_index, bank, tiles_per_row);
        let max_row = (bank.tile_count() / tiles_per_row as usize) + 1;
        // tile_size is already in screen coordinates,
        // so I need to divide by the GUI scale.
        let tiles_height = max_row as f32 * (tile_size / self.gui_scale);

        panel.push_edge(Edge::Top, tiles_height as i16, |tiles| {
            // tiles.set_margin(0);
            // tiles.set_gap(0);
            let rect = tiles.rect();
            let rect_handle = frame_arena.alloc(DrawOp::Rect {
                rect, //
                color: RGBA32 { r: 106, g: 96, b: 128, a: 255 },
            });
            self.ops.push(frame_arena, rect_handle.unwrap()).unwrap();

            let texture_handle = frame_arena
                .alloc(DrawOp::Texture { id: bank_index, rect, tint: RGBA32::WHITE })
                .unwrap();
            self.ops.push(frame_arena, texture_handle).unwrap();

            // Mouse hover detection for tiles
            if rect.contains(mouse.x, mouse.y) {
                let col = ((mouse.x - rect.x) as f32 / tile_size) as i16;
                let row = ((mouse.y - rect.y) as f32 / tile_size) as i16;
                let tile_index = (row * tiles_per_row as i16) + col;
                if tile_index < bank.tile_count() as i16 {
                    self.mouse_over_text =
                        Text::format_display(frame_arena, "Tile {}", &[tile_index], "").unwrap();
                }
            }
        });
    }
}
