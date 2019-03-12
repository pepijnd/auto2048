extern crate rand;
extern crate sdl2;

mod ai;
mod game;
mod ui;

use ui::App;

fn main() {
    let mut app = App::new();
    app.run_app().unwrap();

    // let mut game = game::Game::new();
    // let mut board = game.get_board().clone();

    // let mut ai = ai::AI::new(&board);
    // ai.build_tree(5);
}
