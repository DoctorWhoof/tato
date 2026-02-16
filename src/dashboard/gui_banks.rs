use super::*;

// Right panel
impl Dashboard {
    pub(super) fn process_video_banks_panel<A>(
        &mut self,
        layout: &mut Frame<i16>,
        arena: &mut A,
        banks: &[Bank],
        bg: &dyn DynTilemap,
        backend: &impl Backend,
        tato: &Tato,
    ) where
        A: ArenaOps<u32, ()>,
    {
        layout.push_edge(Edge::Right, PANEL_WIDTH, |right_panel| {
            right_panel.set_margin(5);
            right_panel.set_gap(0);
            right_panel.set_scale(self.gui_scale);
            right_panel.fitting = Fitting::Clamp;

            let rect_handle =
                arena.alloc(DrawOp::Rect { rect: right_panel.rect(), color: DARK_GRAY });
            self.ops.push(arena, rect_handle.unwrap()).unwrap();

            // Process each video memory bank
            for bank_index in 0..BANK_COUNT {
                // Draw each bank debug data
                // self.process_bank(arena, backend, panel, bank_index, tato);
                let Some(bank) = banks.get(bank_index) else { continue };
                self.process_bank(arena, backend, right_panel, bank_index, bank);
                // Small separator
                right_panel.push_edge(Edge::Top, 5, |_separator| {});
            }

            // BG Tile Info
            let size_normal = self.font_size * self.gui_scale;
            // let size_small = self.font_size * 0.9 * self.gui_scale;
            let text = Text::from_str(arena, "BG Tile Info").unwrap();
            self.draw_text(arena, right_panel, text, size_normal);

            right_panel.fill(|tile_info| {
                tile_info.set_gap(0);
                tile_info.set_margin(10);
                self.draw_rect(arena, tile_info.rect(), RGBA12::new(3, 3, 3), false, false);

                if let Some(mouse) = self.world_mouse(backend.get_mouse(), tato) {
                    let tile_size = TILE_SIZE as i16;
                    let col = mouse.x / tile_size;
                    let row = mouse.y / tile_size;

                    if let Some(cell) = bg.get_cell(col, row) {
                        let text = Text::format_display(arena, "TileID: {}", &[cell.id.0], "") //
                            .unwrap();
                        self.draw_text(arena, tile_info, text, size_normal);

                        let text = Text::format_display(
                            arena,
                            "TileFlags:{},{},{},{},{}",
                            &[
                                if cell.flags.is_flipped_x() { "X" } else { "  " },
                                if cell.flags.is_flipped_y() { "Y" } else { "  " },
                                if cell.flags.is_rotated() { "R" } else { "  " },
                                if cell.flags.is_collider() { "C" } else { "  " },
                                if cell.flags.is_fg() { "FG" } else { "  " },
                            ],
                            "",
                        )
                        .unwrap();
                        self.draw_text(arena, tile_info, text, size_normal);

                        let text = Text::format_display(
                            arena,
                            "Colors:{},{},{},{}",
                            &[
                                cell.colors.get(0),
                                cell.colors.get(1),
                                cell.colors.get(2),
                                cell.colors.get(3),
                            ],
                            "",
                        )
                        .unwrap();
                        self.draw_text(arena, tile_info, text, size_normal);

                        // Highlight current BG tile with rect
                        let rect = Rect {
                            x: col * tile_size,
                            y: row * tile_size,
                            w: tile_size,
                            h: tile_size,
                        };
                        self.draw_rect(arena, rect, RGBA12::WHITE, true, true);
                    }
                } else {
                    let text = Text::from_str(arena, "No tile under mouse").unwrap();
                    self.draw_text(arena, tile_info, text, size_normal);
                }
            });
        });
    }

    fn update_tile_texture(&mut self, bank_index: usize, bank: &Bank, tiles_per_row: u16) {
        // Early return for empty banks
        if bank.tiles.count() == 0 {
            // self.tile_pixels[bank_index].clear();
            return;
        }

        // Calculate actual dimensions based on tile layout
        let tile_count = bank.tiles.count() as u16;
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
            // Zero out pixels. If not done there may be garbage from previous tiles
            pixels.fill(0);

            for tile_index in 0..tile_count {
                let tile_x = tile_index % tiles_per_row;
                let tile_y = tile_index / tiles_per_row;

                for y in 0..TILE_SIZE as usize {
                    for x in 0..TILE_SIZE as usize {
                        // get color
                        let color_index =
                            bank.tiles.tiles[tile_index as usize].get_pixel(x as u8, y as u8);
                        // get coordinates
                        let pixel_x = tile_x as usize * TILE_SIZE as usize + x;
                        let pixel_y = tile_y as usize * TILE_SIZE as usize + y;
                        let i = ((pixel_y * w as usize) + pixel_x) * 4;
                        // Seems safe for now, may need to insert a check for i < pixels.len()
                        // if I get out-of-bounds errors.
                        // let color: RGBA32 = bank.colors.palette[color_index as usize].into();
                        pixels[i] = color_index * 85;
                        pixels[i + 1] = color_index * 85;
                        pixels[i + 2] = color_index * 85;
                        pixels[i + 3] = 255;
                    }
                }
            }
        }
    }

    fn process_bank<A>(
        &mut self,
        arena: &mut A,
        backend: &impl Backend,
        panel: &mut Frame<i16>,
        bank_index: usize,
        bank: &Bank,
        // tato: &Tato,
    ) where
        A: ArenaOps<u32, ()>,
    {
        let tiles_per_row = ((TILE_COUNT as f32).sqrt().ceil()) as u16;
        let tile_size = panel.rect().w as f32 / tiles_per_row as f32;

        let gap = self.gui_scale as i16;
        // let Some(bank) = banks[bank_index] else { return; };

        let mouse = backend.get_mouse();

        // Bank label
        let h = self.font_size as i16;
        panel.push_edge(Edge::Top, h, |frame| {
            let rect = frame.rect();
            let text = Text::format_display(arena, "bank: {}", &[bank_index], "").unwrap();
            let handle = arena
                .alloc(DrawOp::Text {
                    text,
                    x: rect.x + gap,
                    y: rect.y,
                    size: self.font_size * self.gui_scale,
                    color: RGBA32::WHITE,
                })
                .unwrap();
            self.ops.push(arena, handle).unwrap();
        });

        // Bank info
        panel.push_edge(Edge::Top, h, |frame| {
            let rect = frame.rect();
            let values = [bank.tiles.count(), bank.colors.color_count() as usize];
            let text = Text::format_display(arena, "{} tiles, {} colors", &values, "").unwrap();

            let handle = arena
                .alloc(DrawOp::Text {
                    text,
                    x: rect.x + gap,
                    y: rect.y,
                    size: self.font_size * 0.75 * self.gui_scale,
                    color: RGBA32::WHITE,
                })
                .unwrap();
            self.ops.push(arena, handle).unwrap();
        });

        if bank.tiles.count() == 0 && bank.colors.color_count() == 0 {
            return;
        }

        // Color palette swatches
        panel.push_edge(Edge::Top, 8, |frame| {
            let rect = frame.rect();
            let rect_handle = arena.alloc(DrawOp::Rect { rect, color: DARKEST_GRAY }).unwrap();
            self.ops.push(arena, rect_handle).unwrap();

            let swatch_w = frame.divide_width(COLORS_PER_PALETTE as u32);
            for c in 0..COLORS_PER_PALETTE as usize {
                frame.push_edge(Edge::Left, swatch_w, |swatch| {
                    let rect = swatch.rect();
                    let color = bank.colors.palette[c];
                    let rgba32 = RGBA32::from(color);

                    let handle = arena.alloc(DrawOp::Rect { rect, color: rgba32 }).unwrap();
                    self.ops.push(arena, handle).unwrap();

                    // Mouse hover detection
                    if rect.contains(mouse.x, mouse.y) {
                        self.mouse_over_text = Text::format_display(
                            arena,
                            "Color {} = {}, {}, {}, {}",
                            &[c as u8, color.r(), color.g(), color.b(), color.a()],
                            "",
                        )
                        .unwrap();
                    }
                });
            }
        });

        // Tile visualization
        self.update_tile_texture(bank_index, bank, tiles_per_row);
        let max_row = (bank.tiles.count() / tiles_per_row as usize) + 1;
        // tile_size is already in screen coordinates,
        // so I need to divide by the GUI scale.
        let tiles_height = max_row as f32 * (tile_size / self.gui_scale);

        panel.push_edge(Edge::Top, tiles_height as i16, |tiles| {
            // tiles.set_margin(0);
            // tiles.set_gap(0);
            let rect = tiles.rect();
            let rect_handle = arena.alloc(DrawOp::Rect {
                rect, //
                color: RGBA32 { r: 106, g: 96, b: 128, a: 255 },
            });
            self.ops.push(arena, rect_handle.unwrap()).unwrap();

            let texture_handle =
                arena.alloc(DrawOp::Texture { id: bank_index, rect, tint: RGBA32::WHITE }).unwrap();
            self.ops.push(arena, texture_handle).unwrap();

            // Mouse hover detection for tiles
            if rect.contains(mouse.x, mouse.y) {
                let col = ((mouse.x - rect.x) as f32 / tile_size) as i16;
                let row = ((mouse.y - rect.y) as f32 / tile_size) as i16;
                let tile_index = (row * tiles_per_row as i16) + col;
                if tile_index < bank.tiles.count() as i16 {
                    self.mouse_over_text =
                        Text::format_display(arena, "Tile {}", &[tile_index], "").unwrap();
                }
            }
        });
    }
}
