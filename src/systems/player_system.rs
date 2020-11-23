///
/// Handles logic for player.
/// 

use crate::{components::PositionComponent, entity_manager::EntityManager, components::LogComponent, input_manager::{self, InputManager}};

// Taken from jdaster64's SMB physics engine guide
const MIN_WALK_VEL: f32 = 0.13;
const MAX_RUN_VEL: f32 = 2.9;
const WALK_ACC: f32 = 0.098;
const RUN_ACC: f32 = 0.144;
const REL_DEC: f32 = 0.13;
const SKID_DEC: f32 = 0.2;
const SKID_SPD: f32 = 0.9;
const JUMP_ACC: f32 = 4.0;
const HOLD_GRAV: f32 = 0.2;
const FALL_GRAV: f32 = 0.7;
const MAX_V_VEL: f32 = 4.8;

// Iterate over entities and update them.
pub fn update(entity_mgr: &mut EntityManager, input_mgr: &InputManager) {
    for entity_id in 0..entity_mgr.entities.len() {
        if entity_mgr.entities[entity_id].exists && entity_mgr.entities[entity_id].use_player {
            update_entity(entity_mgr.get_pos_comp(entity_id as u32),  input_mgr);
        }
    }
}

/// Renders sprites onto screen.
pub fn update_entity(pos_comp: &mut PositionComponent, input_mgr: &InputManager) {
    let in_air = pos_comp.spd_y.abs() > 0.01;

    // Handle horizontal movement
    if input_mgr.is_button_down(input_manager::ButtonCode::LEFT) {
        pos_comp.spd_x -= WALK_ACC;
    }
    else if input_mgr.is_button_down(input_manager::ButtonCode::RIGHT) {
        pos_comp.spd_x += WALK_ACC;
    }
    else {
        // Decelerate or stop moving
        if pos_comp.spd_x.abs() > MIN_WALK_VEL {
            if !in_air {
                pos_comp.spd_x -= pos_comp.spd_x.signum() * REL_DEC;
            }
        }
        else {
            pos_comp.spd_x = 0.0;
        }
    }

    // If maximum running speed has been reached, clip it
    if pos_comp.spd_x.abs() > MAX_RUN_VEL {
        pos_comp.spd_x = pos_comp.spd_x.signum() * MAX_RUN_VEL;
    }

    // Apply gravity
    if input_mgr.is_button_down(input_manager::ButtonCode::A) {
        pos_comp.spd_y += HOLD_GRAV;
    }
    else {
        pos_comp.spd_y += FALL_GRAV;
    }

    // If UP is pressed, apply initial velocity
    if input_mgr.is_button_pressed(input_manager::ButtonCode::A) && !in_air {
        pos_comp.spd_y = -JUMP_ACC;
    }

    // If maximum falling speed has been reached, clip it
    if pos_comp.spd_y > MAX_V_VEL {
        pos_comp.spd_y = MAX_V_VEL;
    }

    // Add speed to position
    pos_comp.prec_x += pos_comp.spd_x;
    pos_comp.prec_y += pos_comp.spd_y;

    // Set up temporary bounds
    let v_offset = 4.0;
    let h_offset = 8.0;
    if pos_comp.prec_y > 196.0 + v_offset {
        pos_comp.prec_y = 196.0 + v_offset;
        pos_comp.spd_y = 0.0;
    }
    if pos_comp.prec_x < 0.0 {
        pos_comp.prec_x = 0.0;
        pos_comp.spd_x = 0.0;
    }
    if pos_comp.prec_x > 256.0 - h_offset {
        pos_comp.prec_x = 256.0 - h_offset;
        pos_comp.spd_x = 0.0;
    }

    // Resolve precise x and y values to actual
    pos_comp.x = pos_comp.prec_x as i32;
    pos_comp.y = pos_comp.prec_y as i32;
}