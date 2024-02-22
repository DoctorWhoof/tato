use crate::*;
use slotmap::new_key_type;
use core::f32::consts::PI;

const COL_MARGIN:f32 = 0.25;

new_key_type! {
    /// A key to the World slotmap containing entities.
    pub struct EntityID;
}

/// Contains the necessary structs to provide rendering and collisions
#[derive(Clone, Debug, Default)]
pub struct Entity {
    // Accessible to "engine" only, not to the host game. Can't be changed after creation.
    pub(super) id: EntityID,

    // Public
    pub depth: u8,      // 0 means "background", higher values render on top
    pub visible: bool, //TODO: Flags? Would allow mirroring whole entity, with colliders and offset properly mirrored.
    pub shape: Shape,
    pub pos: Vec2<f32>,
    pub collider: Option<Collider>,
    pub render_offset: Vec2<i8>,
}

impl Entity {
    pub fn id(&self) -> EntityID {
        self.id
    }

    pub fn new(with_id: EntityID, depth:u8) -> Entity {
        Self {
            depth,
            id: with_id,
            visible: true,
            shape: Shape::None,
            pos: Vec2 { x: 0.0, y: 0.0 },
            render_offset: Vec2 { x: 0, y: 0 },
            collider: None,
        }
    }

    fn world_vec2<T:Into<f32>>(&self, pos: Vec2<T>, use_render_offset: bool) -> Vec2<f32> {
        Vec2 {
            x: if use_render_offset {
                self.pos.x + pos.x.into() + self.render_offset.x as f32
            } else {
                self.pos.x + pos.x.into()
            },
            y: if use_render_offset {
                self.pos.y + pos.y.into() + self.render_offset.y as f32
            } else {
                self.pos.y + pos.y.into()
            },
        }
    }

    pub fn world_rect<T: Into<f32>>(&self, rect: Rect<T>, use_render_offset: bool) -> Rect<f32> {
        let pos = self.world_vec2(
            Vec2 {
                x: rect.x,
                y: rect.y,
            },
            use_render_offset,
        );
        Rect {
            x: pos.x,
            y: pos.y,
            w: rect.w.into(),
            h: rect.h.into(),
        }
    }

    #[allow(clippy::question_mark)]// I'd like to fix this as clippy suggests, but doesn't work?
    pub fn check_collision(&mut self, vel:&mut Vec2<f32>, other:&Self, other_vel:Vec2<f32>) -> Option<Collision<f32>> {
        let Some(col) = &self.collider else { return None };
        let Some(other_col) = &other.collider else { return None };
        match col.kind {
            ColliderKind::Point => {
                match other_col.kind {
                    // Point to point
                    ColliderKind::Point => {
                        if self.pos.floor() == other.pos.floor() {
                            Some(Collision{
                                tile:None,
                                point: self.pos,
                                normal: (-vel.y).atan2(-vel.x), //TODO: "Bounce" off each other, instead of just reversing
                                collider_velocity: other_vel
                            })
                        } else {
                            None
                        }
                    },
                    // Point to Rect
                    ColliderKind::Rect(mut other_rect) => {
                        // Apply position to rect
                        other_rect.x += other.pos.x;
                        other_rect.y += other.pos.y;
                        if other_rect.contains(self.pos.x, self.pos.y){
                            let trajectory = Ray { origin: self.pos, angle: vel.y.atan2(vel.x) + PI };
                            if let Some((col_point, normal)) = other_rect.intersect_ray(&trajectory) {
                                Some(Collision{
                                    tile: None,
                                    point: col_point,
                                    normal,
                                    collider_velocity: other_vel,
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                }
            }
            ColliderKind::Rect(_rect) => {
                None
            }
        }
    }


    pub fn move_and_collide(&mut self, vel:&mut Vec2<f32>, other:&Self, other_vel:Vec2<f32>, elapsed:f32) -> Option<Collision<f32>> {
        
        let prev_pos = self.pos;
        let mut probe_x = self.clone();
        let mut probe_y = self.clone();

        probe_x.pos.x += vel.x * elapsed;
        let result_x = probe_x.check_collision(vel, other, other_vel);
        
        probe_y.pos.y += vel.y * elapsed;
        let result_y = probe_y.check_collision(vel, other, other_vel);

        self.pos.x += vel.x * elapsed;
        self.pos.y += vel.y * elapsed;

        fn set_outgoing_angle(vel:&mut Vec2<f32>, incoming_angle:f32, col:&Collision<f32>) {
            let outgoing_angle = mirror_angle(incoming_angle, col.normal);
            let len = vel.len();
            vel.x = len * outgoing_angle.cos();
            vel.y = len * outgoing_angle.sin();
            // prevents "grabbing" the puck? TODO: Needs testing
            vel.x += col.collider_velocity.x;
            vel.y += col.collider_velocity.y;
        }

        if result_x.is_some() || result_y.is_some() {
            let incoming_angle = self.pos.angle_between(&prev_pos);

            // Resolution in X
            if let Some(col) = &result_x {
                set_outgoing_angle(vel, incoming_angle, col);
                let margin = COL_MARGIN * vel.x.signum();
                self.pos.x = col.point.x + margin;
                return result_x;
            }

            // Resolution in Y
            if let Some(col) = &result_y {
                set_outgoing_angle(vel, incoming_angle, col);
                let margin = COL_MARGIN * vel.y.signum();
                self.pos.y = col.point.y + margin;
                return result_y;
            }

        } 
        None
    }
}
