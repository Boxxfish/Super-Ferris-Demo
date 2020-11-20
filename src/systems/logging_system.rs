///
/// Logs info in LogComponent.
/// 

use crate::components::LogComponent;
use crate::entity_manager::EntityManager;

// Iterate over entities and update them.
pub fn update(entity_mgr: &mut EntityManager) {
    for entity_id in 0..entity_mgr.entities.len() {
        if entity_mgr.entities[entity_id].exists {
            update_entity(entity_mgr.get_log_comp(entity_id as u32));
        }
    }
}

/// Prints out message in log_comp and resets message flag.
pub fn update_entity(log_comp: &mut LogComponent) {
    if log_comp.has_info {
        println!("{}", log_comp.message);
        log_comp.has_info = false;
    }
}