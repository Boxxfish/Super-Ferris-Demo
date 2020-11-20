///
/// Entry point for Super Ferris.
/// 

mod game;
mod entity_manager;
mod components;
mod systems;
mod renderer;

fn main() {
    let mut my_game = game::Game::new();
    my_game.run();
}
