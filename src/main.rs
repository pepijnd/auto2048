extern crate rand;
extern crate rsrl;
extern crate sdl2;

#[macro_use]
extern crate slog;

use rand::prelude::*;
use rand::{thread_rng, Rng};

mod ai;
mod game;
mod learning;
mod ui;

use ui::App;

use crate::learning::Learning;

use crate::ai::AIScore;
use crate::ai::AI;
use crate::game::{Board, Direction, Game};

use std::env;
fn main() {
    let mut args = env::args().into_iter();
    let target = args.nth(1).unwrap_or("gui".to_string());
    if target == "gui" {
        let app = App::new();
        app.run_app().unwrap();
    } else if target == "bench" {
        let mut game = Game::new();
        let mut run = true;
        let mut steps = 0;
        let mut won = false;

        while run {
            //let mut ai = AI::new(&game.get_board(), 6);
            let mut ai = AI::new(&game.get_board(), 6);
            ai.build_tree();
            let minimax = ai.minimax(None);
            if game.step(minimax.get_direction()) == false || game.has_won() {
                won = game.has_won();
                run = false;
            }
            steps += 1;
        }
        if won {
            println!("game won with a score of {}", game.get_score());
        } else {
            println!("game lost with a score of {}", game.get_score());
        }
    } else if target == "learn" {
        Learning::learn();
    } else if target == "rand" {
        let mut game = Game::new();
        let mut run = true;
        let mut steps = 0;
        let mut won = false;

        let mut scores = Vec::new();

        let n = 100000000;
        for _ in 0..n {
            while run {
                //let mut ai = AI::new(&game.get_board(), 6);
                let mut ai = AI::new(&game.get_board(), 6);
                ai.build_tree();
                let minimax = ai.minimax(Some(Box::new(move |e: &Board| {
                    let mut rng = thread_rng();
                    rng.gen_range(-10f64, 10f64)
                })));
                if game.step(minimax.get_direction()) == false || game.has_won() {
                    won = game.has_won();
                    run = false;
                }
                steps += 1;
            }
            if won {
                scores.push(1f64);
            } else {
                scores.push(game.get_score() as f64 / 3072f64);
            }
        }
        println!(
            "{} avg after {} games",
            scores.iter().sum::<f64>() / scores.len() as f64,
            scores.len()
        );
    }
}
