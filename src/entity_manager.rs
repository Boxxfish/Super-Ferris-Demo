///
/// Manages game entities.
/// 

use std::vec;
use crate::components::{PositionComponent, SpriteComponent};

use super::components::LogComponent;
use super::components::Component;

/// Automates adding a new component to the manager.
#[macro_export]
macro_rules! setup_comp {
    ($comp_type:ident, $entity_comps_name:ident, $comp_ind_name:ident, $add_name:ident, $get_name:ident, $get_name_immut:ident, $index_name:ident) => {
        /// Adds a component to the entity.
        pub fn $add_name(&mut self, entity_id: u32) {
            let ind = self.$index_name();
            self.$entity_comps_name[ind as usize].exists = true;
            self.entities[entity_id as usize].$comp_ind_name = ind;
        }

        /// Returns an entity's component.
        pub fn $get_name(&mut self, entity_id: u32) -> &mut $comp_type {
            let ind = self.entities[entity_id as usize].$comp_ind_name;
            return &mut self.$entity_comps_name[ind as usize];
        }

        /// Returns an entity's component immutably.
        pub fn $get_name_immut(&self, entity_id: u32) -> &$comp_type {
            let ind = self.entities[entity_id as usize].$comp_ind_name;
            return &self.$entity_comps_name[ind as usize];
        }
        
        /// Returns the next free index in the component list.
        /// May expand component list.
        fn $index_name(&mut self) -> u32 {
            match self.$entity_comps_name.iter().find(|&comp| {!comp.exists}) {
                Some(comp) => comp.id,
                None => {
                    self.$entity_comps_name.push($comp_type::uninit());
                    self.$entity_comps_name.len() as u32 - 1
                },
            }
        }
    };
}

pub struct EntityManager {
    pub entities: Vec<Entity>,
    log_comps: Vec<LogComponent>,
    sprite_comps: Vec<SpriteComponent>,
    pos_comps: Vec<PositionComponent>,
}

const INITIAL_ENTITIES_LEN: usize = 32;
const INITIAL_COMPS_LEN: usize = 32;

impl EntityManager {
    // TODO: This is an improvement, but make it less wordy
    setup_comp!(LogComponent, log_comps, log_ind, add_log_comp, get_log_comp, get_log_comp_immut, get_next_free_log_index);
    setup_comp!(SpriteComponent, sprite_comps, sprite_ind, add_sprite_comp, get_sprite_comp, get_sprite_comp_immut, get_next_free_sprite_index);
    setup_comp!(PositionComponent, pos_comps, pos_ind, add_pos_comp, get_pos_comp, get_pos_comp_immut, get_next_free_pos_index);

    pub fn new() -> Self {
        // Component lists should have at least 1 element
        // Index 0 has the null component
        let mut log_comps = vec![
            LogComponent::uninit()
        ];
        log_comps.reserve(INITIAL_COMPS_LEN - 1);
        let mut sprite_comps = vec![
            SpriteComponent::uninit()
        ];
        sprite_comps.reserve(INITIAL_COMPS_LEN - 1);
        let mut pos_comps = vec![
            PositionComponent::uninit()
        ];
        pos_comps.reserve(INITIAL_COMPS_LEN - 1);

        EntityManager {
            entities: vec::Vec::with_capacity(INITIAL_ENTITIES_LEN),
            log_comps,
            sprite_comps,
            pos_comps
        }
    }

    /// Creates a new entity and returns its ID.
    pub fn create_entity(&mut self) -> u32 {
        // Create a new Entity entry
        // By default, has no components
        let entity_id = self.get_next_free_id();
        let mut entity = Entity {
            id: entity_id,
            exists: true,
            log_ind: 0,
            sprite_ind: 0,
            pos_ind: 0,
            use_draw: false,
            use_log: false,
            use_player: false,
        };
        self.entities[entity_id as usize] = entity;

        entity_id
    }

    /// Returns the next free ID.
    /// May expand entity list.
    fn get_next_free_id(&mut self) -> u32 {
        match self.entities.iter().find(|&&entity| {!entity.exists}) {
            Some(entity) => entity.id,
            None => {
                self.entities.push(Entity::uninit());
                self.entities.len() as u32 - 1
            },
        }
    }

    /// Activates the logging system for the entity.
    pub fn set_use_log(&mut self, entity_id: u32) {
        self.entities[entity_id as usize].use_log = true;
    }

    /// Activates the draw system for the entity.
    pub fn set_use_draw(&mut self, entity_id: u32) {
        self.entities[entity_id as usize].use_draw = true;
    }

    /// Activates the player system for the entity.
    pub fn set_use_player(&mut self, entity_id: u32) {
        self.entities[entity_id as usize].use_player = true;
    }
}

/// Contains indices for components.
#[derive(Debug, Copy, Clone)]
pub struct Entity {
    pub id: u32,
    pub exists: bool,
    log_ind: u32,
    sprite_ind: u32,
    pos_ind: u32,
    pub use_log: bool,
    pub use_draw: bool,
    pub use_player: bool
}

impl Entity {
    /// Creates an uninitialized entity.
    fn uninit() -> Self {
        Self {
            id: 0,
            exists: false,
            log_ind: 0,
            sprite_ind: 0,
            pos_ind: 0,
            use_log: false,
            use_draw: false,
            use_player: false
        }
    }
}