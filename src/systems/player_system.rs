///
/// Handles logic for player.
/// 

use crate::{components::PositionComponent, entity_manager::EntityManager, components::LogComponent, input_manager::{self, InputManager}};

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
    if input_mgr.is_button_down(input_manager::ButtonCode::LEFT) {
        pos_comp.x -= 1;
    }
    if input_mgr.is_button_down(input_manager::ButtonCode::RIGHT) {
        pos_comp.x += 1;
    }
    if input_mgr.is_button_down(input_manager::ButtonCode::UP) {
        pos_comp.y -= 1;
    }
    if input_mgr.is_button_down(input_manager::ButtonCode::DOWN) {
        pos_comp.y += 1;
    }
}