use crate::*;

// "Character Controller" function, can be used on any entity.
pub fn character_control(input:&Input, id:EntityID, groups:&[TileKind], char_state:&mut CharState, world:&mut World, tilemap_id:EntityID){

    let time = world.time;
    let elapsed = world.time_elapsed();
    let limit = world.get_entity_rect_from_id(tilemap_id);

    // Probes
    let entity = world.get_entity(id);
    char_state.down_tile.check_tile(world.tile_at(entity.pos.x, entity.pos.y, tilemap_id), groups);
    char_state.left_tile.check_tile(world.tile_at(entity.pos.x - 8.0, entity.pos.y - 8.0, tilemap_id), groups);
    char_state.right_tile.check_tile(world.tile_at(entity.pos.x + 8.0, entity.pos.y - 8.0, tilemap_id), groups);
    // If first up_tile is none, run a second one a little above to account for props like the telephone
    char_state.up_tile.check_tile(world.tile_at(entity.pos.x, entity.pos.y - 16.0, tilemap_id), groups);
    if let TileKind::None = char_state.up_tile.kind {
        char_state.up_tile.check_tile(world.tile_at(entity.pos.x, entity.pos.y - 24.0, tilemap_id), groups);
    }

    // We now need mutable refs to world entities and anims.
    let (entities, anims, _) = world.get_data_mut();
    let entity = &mut entities[id.get()];
    let Shape::Sprite{ref mut flip_h, ref mut anim_id, ..} = entity.shape else { return };

    // Reset state
    char_state.vel.x = 0.0;
    char_state.vel.y = 0.0;
    char_state.action = Action::Idle;

    // Set velocity based on floor type
    let speed = char_state.speed as f32;
    let diagonal_speed = speed;// * 0.75;


    // let (x_vel, y_vel) = (speed, 0.0);
    let (x_vel, y_vel) = match char_state.down_tile.kind {
        TileKind::Stairs => if char_state.down_tile.tile.flipped_h() {
            (diagonal_speed, -diagonal_speed)
        } else {
            (diagonal_speed, diagonal_speed)
        },
        _ => (speed, 0.0)
    };

    // Horizontal movement
    if input.left && char_state.left_tile.kind != TileKind::Wall {
        char_state.vel.x = - x_vel;
        char_state.vel.y = y_vel;
        char_state.action = Action::Moving;
        *flip_h = true;
    } else if input.right && char_state.right_tile.kind != TileKind::Wall{
        char_state.vel.x = x_vel;
        char_state.vel.y = -y_vel;
        char_state.action = Action::Moving;
        *flip_h = false;
    }

    // if input.up { //&& char_state.left_tile.kind != TileKind::Wall {
    //     char_state.vel.y = -60.0;
    //     char_state.action = Action::Moving;
    // } else if input.down {// && char_state.right_tile.kind != TileKind::Wall{
    //     char_state.vel.y = 60.0;
    //     char_state.action = Action::Moving;
    // }


    // Apply velocity
    entity.pos.x += char_state.vel.x * elapsed;
    entity.pos.y += char_state.vel.y * elapsed;

    // Quantize height to floor (after vel is applied)
    if char_state.down_tile.kind != TileKind::Stairs {
        entity.pos.y = quantize(entity.pos.y, CEILING_HEIGHT) - 6.0
    }

    // Wrap around
    if entity.pos.x < limit.x { entity.pos.x = limit.right() - 1.0 }
    if entity.pos.x > limit.right() { entity.pos.x = limit.x + 1.0 }

    // Process animation
    if char_state.action == Action::Moving {
        *anim_id = char_state.anim_run;
        // Up and down when running
        let anim = &anims[anim_id.get()];
        entity.render_offset.y = -24 -[1,1,0,-1,0][anim.current_frame_number(time)];
    } else {
        *anim_id = char_state.anim_idle;
        entity.render_offset.y = -24
    }
}
