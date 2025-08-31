use super::*;

impl Dashboard {
    pub fn draw_polys<const LEN: usize>(
        &mut self,
        frame_arena: &mut Arena<LEN>,
        tato: &Tato,
        // backend: &impl Backend,
    ) {
        // Generate ops for debug polygons
        for all_polys in self.debug_polys_gui.items(&self.debug_arena).unwrap() {
            let poly = self.debug_arena.get_slice(all_polys).unwrap();
            if poly.len() >= 2 {
                for i in 0..(poly.len() - 1) {
                    let current = poly[i];
                    let next = poly[i + 1];
                    let handle = frame_arena
                        .alloc(DrawOp::Line {
                            x1: current.x,
                            y1: current.y,
                            x2: next.x,
                            y2: next.y,
                            color: RGBA32::WHITE,
                        })
                        .unwrap();
                    self.ops.push(frame_arena, handle).expect("Dashboard: Can't insert GUI poly");
                }
            }
        }

        // World space polys (will follow scrolling)\
        if let Some(canvas_rect) = self.canvas_rect {
            let video_size = tato.video.size();
            for all_polys in self.debug_polys_world.items(&self.debug_arena).unwrap() {
                let world_poly = self.debug_arena.get_slice(all_polys).unwrap();
                let scale = canvas_rect.h as f32 / video_size.y as f32;
                let scroll_x = tato.video.scroll_x as f32;
                let scroll_y = tato.video.scroll_y as f32;
                if world_poly.len() >= 2 {
                    for i in 0..(world_poly.len() - 1) {
                        let current = world_poly[i];
                        let next = world_poly[i + 1];
                        let handle = frame_arena
                            .alloc(DrawOp::Line {
                                x1: ((current.x as f32 - scroll_x) * scale) as i16 + canvas_rect.x,
                                y1: ((current.y as f32 - scroll_y) * scale) as i16 + canvas_rect.y,
                                x2: ((next.x as f32 - scroll_x) * scale) as i16 + canvas_rect.x,
                                y2: ((next.y as f32 - scroll_y) * scale) as i16 + canvas_rect.y,
                                color: RGBA32::WHITE,
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
