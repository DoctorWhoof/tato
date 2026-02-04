use super::*;

impl Dashboard {
    pub(super) fn draw_polys<A>(&mut self, frame_arena: &mut A, tato: &Tato)
    where
        A: ArenaOps<u32, ()>,
    {
        // Generate ops for debug polygons
        // Using index-based iteration to avoid holding the arena borrow
        let gui_polys_len = self.debug_polys_gui.len();
        for i in 0..gui_polys_len {
            // Get polygon at index i - borrow is released immediately after copying
            let Some(poly) = self.debug_polys_gui.get(frame_arena, i) else {
                break;
            };

            // Get polygon length without holding the slice
            let poly_len = frame_arena.get_slice(poly.points).unwrap().len();

            if poly_len >= 2 {
                for j in 0..(poly_len - 1) {
                    let points = frame_arena.get_slice(poly.points).unwrap();
                    let current = points[j];
                    let next = points[j + 1];

                    let handle = if poly.clip_to_view && self.canvas_rect.is_some() {
                        // TODO: Intersect line with rect to find where it starts inside
                        // view, instead of clamping
                        let Some(rect) = self.canvas_rect else {
                            continue;
                        };
                        if (current.x < rect.left() && next.x < rect.left())
                            || (current.x > rect.right() && next.x > rect.right())
                            || (current.y < rect.top() && next.y < rect.top())
                            || (current.y > rect.bottom() && next.y > rect.bottom())
                        {
                            continue;
                        }
                        let x1 = current.x.clamp(rect.left(), rect.right() - 2);
                        let y1 = current.y.clamp(rect.top(), rect.bottom() - 2);
                        let x2 = next.x.clamp(rect.left(), rect.right() - 2);
                        let y2 = next.y.clamp(rect.top(), rect.bottom() - 2);
                        frame_arena
                            .alloc(DrawOp::Line { x1, y1, x2, y2, color: poly.color.into() })
                            .unwrap()
                    } else {
                        frame_arena
                            .alloc(DrawOp::Line {
                                x1: current.x,
                                y1: current.y,
                                x2: next.x,
                                y2: next.y,
                                color: poly.color.into(),
                            })
                            .unwrap()
                    };
                    self.ops.push(frame_arena, handle).expect("Dashboard: Can't insert GUI poly");
                }
            }
        }

        // World space polys (will follow scrolling)
        if let Some(canvas_rect) = self.canvas_rect {
            let video_size = tato.video.size();
            let scale = canvas_rect.h as f32 / video_size.y as f32;

            // Using index-based iteration to avoid holding the arena borrow
            let world_polys_len = self.debug_polys_world.len();
            for i in 0..world_polys_len {
                // Get polygon at index i - borrow is released immediately after copying
                let Some(poly) = self.debug_polys_world.get(frame_arena, i) else {
                    break;
                };

                // Get polygon length without holding the slice
                let poly_len = frame_arena.get_slice(poly.points).unwrap().len();

                if poly_len >= 2 {
                    for j in 0..(poly_len - 1) {
                        let points = frame_arena.get_slice(poly.points).unwrap();
                        let current = points[j];
                        let next = points[j + 1];

                        let a = world_to_view(current, tato.video.scroll, canvas_rect, scale);
                        let b = world_to_view(next, tato.video.scroll, canvas_rect, scale);

                        let handle = if poly.clip_to_view {
                            if (a.x < canvas_rect.left() && b.x < canvas_rect.left())
                                || (a.x > canvas_rect.right() && b.x > canvas_rect.right())
                                || (a.y < canvas_rect.top() && b.y < canvas_rect.top())
                                || (a.y > canvas_rect.bottom() && b.y > canvas_rect.bottom())
                            {
                                continue;
                            }
                            // TODO: Intersect line with rect to find where it starts inside
                            // view, instead of clamping
                            let x1 = a.x.clamp(canvas_rect.left(), canvas_rect.right() - 2);
                            let y1 = a.y.clamp(canvas_rect.top(), canvas_rect.bottom() - 2);
                            let x2 = b.x.clamp(canvas_rect.left(), canvas_rect.right() - 2);
                            let y2 = b.y.clamp(canvas_rect.top(), canvas_rect.bottom() - 2);
                            frame_arena
                                .alloc(DrawOp::Line { x1, y1, x2, y2, color: poly.color.into() })
                                .unwrap()
                        } else {
                            frame_arena
                                .alloc(DrawOp::Line {
                                    x1: a.x,
                                    y1: a.y,
                                    x2: b.x,
                                    y2: b.y,
                                    color: poly.color.into(),
                                })
                                .unwrap()
                        };
                        self.ops
                            .push(frame_arena, handle)
                            .expect("Dashboard: Can't insert World poly");
                    }
                }
            }
        }
    }
}
