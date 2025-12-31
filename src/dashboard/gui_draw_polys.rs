use super::*;

impl Dashboard {
    pub(super) fn draw_polys<A>(&mut self, frame_arena: &mut A, tato: &Tato)
    where
        A: ArenaOps<u32, ()>,
    {
        // Generate ops for debug polygons
        // Use index-based iteration to avoid holding the arena borrow
        let gui_polys_len = self.debug_polys_gui.len();
        for i in 0..gui_polys_len {
            // Get polygon at index i - borrow is released immediately after copying
            let Some(poly) = self.debug_polys_gui.get(frame_arena, i) else {
                break;
            };

            // Get color once - this is a simple value copy
            let color: RGBA32 = frame_arena.get(poly.color).copied().unwrap().into();

            // Get polygon length without holding the slice
            let poly_len = frame_arena.get_slice(poly.points).unwrap().len();

            if poly_len >= 2 {
                for j in 0..(poly_len - 1) {
                    // Get points individually - brief borrows that are released immediately
                    let current = frame_arena.get_slice(poly.points).unwrap()[j];
                    let next = frame_arena.get_slice(poly.points).unwrap()[j + 1];

                    // Now we can allocate without any conflicting borrows
                    let handle = frame_arena
                        .alloc(DrawOp::Line {
                            x1: current.x,
                            y1: current.y,
                            x2: next.x,
                            y2: next.y,
                            color,
                        })
                        .unwrap();
                    self.ops.push(frame_arena, handle).expect("Dashboard: Can't insert GUI poly");
                }
            }
        }

        // World space polys (will follow scrolling)
        if let Some(canvas_rect) = self.canvas_rect {
            let video_size = tato.video.size();
            let scale = canvas_rect.h as f32 / video_size.y as f32;
            let scroll_x = tato.video.scroll_x as f32;
            let scroll_y = tato.video.scroll_y as f32;

            // Use index-based iteration to avoid holding the arena borrow
            let world_polys_len = self.debug_polys_world.len();
            for i in 0..world_polys_len {
                // Get polygon at index i - borrow is released immediately after copying
                let Some(poly) = self.debug_polys_world.get(frame_arena, i) else {
                    break;
                };

                // Get color once - this is a simple value copy
                let color: RGBA32 = frame_arena.get(poly.color).copied().unwrap().into();

                // Get polygon length without holding the slice
                let poly_len = frame_arena.get_slice(poly.points).unwrap().len();

                if poly_len >= 2 {
                    for j in 0..(poly_len - 1) {
                        // Get points individually - brief borrows that are released immediately
                        let current = frame_arena.get_slice(poly.points).unwrap()[j];
                        let next = frame_arena.get_slice(poly.points).unwrap()[j + 1];

                        // Now we can allocate without any conflicting borrows
                        let handle = frame_arena
                            .alloc(DrawOp::Line {
                                x1: ((current.x as f32 - scroll_x) * scale) as i16 + canvas_rect.x,
                                y1: ((current.y as f32 - scroll_y) * scale) as i16 + canvas_rect.y,
                                x2: ((next.x as f32 - scroll_x) * scale) as i16 + canvas_rect.x,
                                y2: ((next.y as f32 - scroll_y) * scale) as i16 + canvas_rect.y,
                                color,
                            })
                            .unwrap();
                        self.ops
                            .push(frame_arena, handle)
                            .expect("Dashboard: Can't insert World poly");
                    }
                }
            }
        }
    }
}
