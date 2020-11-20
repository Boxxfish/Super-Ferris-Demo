///
/// Manages game entities.
/// 

use std::vec;
use super::components::LogComponent;

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

    /// Adds a log component to the entity.
    // TODO: Automate this
    pub fn add_log_comp(&mut self, entity_id: u32) {
        let log_ind = self.get_next_free_log_index();
        self.log_comps[log_ind as usize].exists = true;
        self.entities[entity_id as usize].log_ind = log_ind;
    }

    /// Returns an entity's log component.
    // TODO: Automate this
    pub fn get_log_comp(&mut self, entity_id: u32) -> &mut LogComponent {
        let log_ind = self.entities[entity_id as usize].log_ind;
        return &mut self.log_comps[log_ind as usize];
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
    
    /// Returns the next free index in the log component list.
    /// May expand log component list.
    // TODO: Create a macro that automates the whole component adding system
    fn get_next_free_log_index(&mut self) -> u32 {
        match self.log_comps.iter().find(|&comp| {!comp.exists}) {
            Some(comp) => comp.id,
            None => {
                self.log_comps.push(LogComponent::uninit());
                self.log_comps.len() as u32 - 1
            },
        }
    }
}