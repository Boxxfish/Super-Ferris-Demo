///
/// Manages game entities.
/// 

use std::vec;
use super::components::LogComponent;
use super::components::Component;

/// Automates adding a new component to the manager.
#[macro_export]
macro_rules! setup_comp {
    ($comp_type:ident, $entity_comps_name:ident, $comp_ind_name:ident, $add_name:ident, $get_name:ident, $index_name:ident) => {
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
        
        /// Returns the next free index in the component list.
        /// May expand component list.
        fn $index_name(&mut self) -> u32 {
            match self.$entity_comps_name.iter().find(|&comp| {!comp.exists}) {
                Some(comp) => comp.id,
                None => {
                    self.log_comps.push($comp_type::uninit());
                    self.log_comps.len() as u32 - 1
                },
            }
        }
    };
}

/// Contains indices for components.
#[derive(Debug, Copy, Clone)]
pub struct Entity {
    pub id: u32,
    pub exists: bool,
    log_ind: u32
}

impl Entity {
    /// Creates an uninitialized entity.
    fn uninit() -> Self {
        Self {
            id: 0,
            exists: false,
            log_ind: 0
        }
    }
}

pub struct EntityManager {
    pub entities: Vec<Entity>,
    log_comps: Vec<LogComponent>
}

const INITIAL_ENTITIES_LEN: usize = 32;
const INITIAL_COMPS_LEN: usize = 32;

impl EntityManager {
    // TODO: This is an improvement, but make it less wordy
    setup_comp!(LogComponent, log_comps, log_ind, add_log_comp, get_log_comp, get_next_free_log_index);

    pub fn new() -> Self {
        // Component lists should have at least 1 element
        // Index 0 has the null component
        let mut log_comps = vec![
            LogComponent {
                id: 0,
                exists: true,
                has_info: false,
                message: String::from("")
            }
        ];
        log_comps.reserve(INITIAL_COMPS_LEN - 1);

        EntityManager {
            entities: vec::Vec::with_capacity(INITIAL_ENTITIES_LEN),
            log_comps
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
            log_ind: 0
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
}