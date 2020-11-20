///
/// Manages execution of the game.
///

use super::entity_manager;
use super::systems::*;

pub struct Game {
    entity_mgr: entity_manager::EntityManager
}

impl Game {
    pub fn new() -> Self {
        Game {
            entity_mgr: entity_manager::EntityManager::new()
        }
    }

    /// Sets up everything and runs until game stops.
    pub fn run(&self) {
        // Do setup here
        let mut should_run = true;

        // Game loop
        while should_run {

            // Quit for now
            should_run = false;
        }
    }
}

/// Test if logging works on a single entity.
#[test]
fn test_logging_entity() {
    let mut entity_mgr = entity_manager::EntityManager::new();
    let entity_id = entity_mgr.create_entity();
    entity_mgr.add_log_comp(entity_id);
    
    let log_comp = entity_mgr.get_log_comp(entity_id);
    log_comp.message = String::from("Logging test.");
    log_comp.has_info = true;
    
    logging_system::update_entity(log_comp);
    assert_eq!(log_comp.has_info, false);
}

/// Test if logging works across the entire entity manager.
#[test]
fn test_logging_entities() {
    let mut entity_mgr = entity_manager::EntityManager::new();
    let entity_id = entity_mgr.create_entity();
    entity_mgr.add_log_comp(entity_id);

    let log_comp = entity_mgr.get_log_comp(entity_id);
    log_comp.message = String::from("Logging test.");
    log_comp.has_info = true;
    
    logging_system::update(&mut entity_mgr);
    assert_eq!(entity_mgr.get_log_comp(entity_id).has_info, false);
}