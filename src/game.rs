///
/// Manages execution of the game.
///

use std::sync;

use winit::{dpi, event_loop};
use winit::event;
use winit::window;
use timer;

use crate::systems::{draw_system, logging_system, player_system};

use super::entity_manager;
use super::input_manager;
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

        // Set up input manager
        let input_mgr_mut = sync::Arc::new(sync::Mutex::new(input_manager::InputManager::new()));
        let mut input_mgr = input_mgr_mut.lock().unwrap();
        input_mgr.map_key_to_button(event::VirtualKeyCode::Left, input_manager::ButtonCode::LEFT);
        input_mgr.map_key_to_button(event::VirtualKeyCode::Right, input_manager::ButtonCode::RIGHT);
        input_mgr.map_key_to_button(event::VirtualKeyCode::Up, input_manager::ButtonCode::UP);
        input_mgr.map_key_to_button(event::VirtualKeyCode::Down, input_manager::ButtonCode::DOWN);
        input_mgr.map_key_to_button(event::VirtualKeyCode::X, input_manager::ButtonCode::A);
        input_mgr.map_key_to_button(event::VirtualKeyCode::Z, input_manager::ButtonCode::B);
        drop(input_mgr);

        // Set up game framework
        let entity_mgr_mut = sync::Arc::new(sync::Mutex::new(entity_manager::EntityManager::new()));
        let mut renderer = futures::executor::block_on(renderer::Renderer::new(&window));

        // Set up entities
        let mut entity_mgr = entity_mgr_mut.lock().unwrap();
        let entity_id = entity_mgr.create_entity();
        entity_mgr.set_use_draw(entity_id);
        entity_mgr.set_use_player(entity_id);
        entity_mgr.add_pos_comp(entity_id);
        entity_mgr.add_sprite_comp(entity_id);
        entity_mgr.get_sprite_comp(entity_id).tex_name = String::from("assets/ferris.png");

        // Load level
        let level_str = include_str!("../assets/level.txt");
        let mut y = 0;
        let mut x = 0;
        for c in level_str.chars() {
            match c {
                '\n' => {y += 1; x = 0},
                'x' => {
                    let block_id = entity_mgr.create_entity();
                    entity_mgr.set_use_draw(block_id);
                    entity_mgr.add_pos_comp(block_id);
                    entity_mgr.add_sprite_comp(block_id);
                    entity_mgr.get_sprite_comp(block_id).tex_name = String::from("assets/tileset.png");
                    entity_mgr.get_sprite_comp(block_id).sprite_index = 2;
                    entity_mgr.get_pos_comp(block_id).x = x * 16;
                    entity_mgr.get_pos_comp(block_id).y = y * 16;
                    x += 1
                },
                _ => x += 1
            }
        }
        drop(entity_mgr);

        // Start game logic thread
        // Runs once per 1/60'th of a second.
        let timer = timer::Timer::new();
        let entity_mgr_mut_ref = sync::Arc::clone(&entity_mgr_mut);
        let input_mgr_mut_ref = sync::Arc::clone(&input_mgr_mut);
        let guard = timer.schedule_repeating(chrono::Duration::milliseconds(16), move || {
            let mut entity_mgr = entity_mgr_mut_ref.lock().unwrap();
            let mut input_mgr = input_mgr_mut_ref.lock().unwrap();
            logging_system::update(&mut entity_mgr);
            player_system::update(&mut entity_mgr, &input_mgr);
        });

        // Start event loop
        let entity_mgr_mut_ref = sync::Arc::clone(&entity_mgr_mut);
        let input_mgr_mut_ref = sync::Arc::clone(&input_mgr_mut);
        evt_loop.run(move |event, _, control_flow| {
            match event {
                event::Event::WindowEvent {
                    window_id,
                    event
                } if window_id == window.id() => match event {
                    // If window should close, exit
                    event::WindowEvent::CloseRequested => {*control_flow = event_loop::ControlFlow::Exit},
                    // If keyboard input is detected, handle it
                    event::WindowEvent::KeyboardInput { 
                        input,
                        ..
                    } => {
                        let mut input_mgr = input_mgr_mut_ref.lock().unwrap();

                        let v_key_code = input.virtual_keycode;
                        if v_key_code.is_none() {
                            return;
                        }

                        let button = input_mgr.key_to_button(v_key_code.unwrap());
                        if button.is_some() {
                            let button = button.unwrap();
                            if input.state == event::ElementState::Pressed {
                                input_mgr.set_button_pressed(button);
                            }
                            if input.state == event::ElementState::Released {
                                input_mgr.set_button_released(button);
                            }
                        }
                    },
                    _ => {}
                },
                // If all events were handled, update and render
                event::Event::MainEventsCleared => {
                    let mut entity_mgr = entity_mgr_mut_ref.lock().unwrap();
                    draw_system::update(&mut entity_mgr, &mut renderer);
                    renderer.render();
                },
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
    entity_mgr.set_use_log(entity_id);
    
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
    entity_mgr.set_use_log(entity_id);

    let log_comp = entity_mgr.get_log_comp(entity_id);
    log_comp.message = String::from("Logging test.");
    log_comp.has_info = true;
    
    logging_system::update(&mut entity_mgr);
    assert_eq!(entity_mgr.get_log_comp(entity_id).has_info, false);
}