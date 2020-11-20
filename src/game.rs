///
/// Manages execution of the game.
///

use winit::{dpi, event_loop};
use winit::event;
use winit::window;

use super::entity_manager;
use super::systems::logging_system;
use super::renderer;

pub struct Game {

}

impl Game {
    pub fn new() -> Self {
        Game {
            
        }
    }

    /// Sets up everything and runs until game stops.
    pub fn run(&mut self) {
        // Create event loop and window
        let evt_loop = event_loop::EventLoop::new();
        let window = window::WindowBuilder::new()
            .with_title("Super Ferris")
            .with_resizable(false)
            .with_inner_size(dpi::PhysicalSize {
                width: renderer::WIN_WIDTH,
                height: renderer::WIN_HEIGHT
            })
            .build(&evt_loop)
            .expect("Could not create window.");

        let entity_mgr = entity_manager::EntityManager::new();
        let mut renderer = futures::executor::block_on(renderer::Renderer::new(&window));
        
        // Start event loop
        evt_loop.run(move |event, _, control_flow| {
            match event {
                // If window should close, exit
                event::Event::WindowEvent {
                    window_id,
                    event: event::WindowEvent::CloseRequested
                } if window_id == window.id() => *control_flow = event_loop::ControlFlow::Exit,
                // If all events were handled, render
                event::Event::MainEventsCleared => renderer.render(),
                // Otherwise, do nothing
                _ => ()
            }
        });
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