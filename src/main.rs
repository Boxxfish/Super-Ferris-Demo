///
/// Entry point for Super Ferris.
/// 

mod game;
mod entity_manager;
mod components;
mod systems;

fn main() {
    let my_game = game::Game::new();
    my_game.run();
}
