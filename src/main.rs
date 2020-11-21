///
/// Entry point for Super Ferris.
/// 

mod game;
mod entity_manager;
mod renderer;
mod input_manager;
mod components;
mod systems;

fn main() {
    let mut my_game = game::Game::new();
    my_game.run();
}
