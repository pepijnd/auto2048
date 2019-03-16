extern crate rand;
extern crate sdl2;
extern crate serde;
extern crate serde_json;

extern crate rustneat;

use rustneat::Environment;
use rustneat::Organism;
use rustneat::Population;

#[cfg(feature = "telemetry")]
extern crate open;
#[cfg(feature = "telemetry")]
extern crate rusty_dashed;
#[cfg(feature = "telemetry")]
use self::rusty_dashed::Dashboard;

mod ai;
mod game;
mod ui;

use ui::App;

use crate::ai::AIScore;
use crate::ai::AI;
use crate::game::{Board, Direction, Game};

use serde::{Serialize, Deserialize};
use serde_json::*;


use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use std::fs::File;
use std::fs::create_dir_all;
use std::io::prelude::*;
use std::path::Path;

extern crate chrono;
use chrono::prelude::*;

static mut local: Option<DateTime<Local>> = None;

fn output_json(data: String, gen: u32) {
    let folder = format!("genomes/{}/", unsafe{local.unwrap().to_string()});
    create_dir_all(&folder);
    let name = format!("genomes/{}/{}.json", unsafe{local.unwrap().to_string()}, gen);
    let path = Path::new(&name);
    let mut file = File::create(&path).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

#[cfg(feature = "telemetry")]
pub fn enable_telemetry(query_string: &str) {
    let mut dashboard = Dashboard::new();
    dashboard.add_graph("fitness1", "fitness", 0, 0, 4, 4);
    dashboard.add_graph("network1", "network", 4, 0, 4, 4);

    rusty_dashed::Server::serve_dashboard(dashboard);

    let url = format!("http://localhost:3000{}", query_string);
    match open::that(url.clone()) {
        Err(_) => println!(
            "\nOpen browser and go to {:?} to see how neural network evolves\n",
            url
        ),
        _ => println!("Openning browser..."),
    }
    std::thread::sleep(std::time::Duration::new(1, 0));
}

struct GameEnv;

impl Environment for GameEnv {
    fn test(&self, organism: &mut Organism) -> f64 {
        let mut game = Game::new();
        let mut run = true;
        let mut steps = 0;
        let mut won = false;

        loop {
            let organism = Rc::new(RefCell::new(organism.clone()));
            let mut ai = AI::new(&game.get_board(), 6);
            ai.build_tree();
            let minimax = ai.minimax(Some(Box::new(move |e: &Board| {
                let mut cells = 0;
                let mut input = Vec::new();
                let mut output = vec![0f64];

                for y in 0..e.height() {
                    for x in 0..e.width() {
                        let cell = e.get_cell(x, y);
                        if cell.is_set() {
                            let cell_score = cell.get_score().unwrap();
                            let score = (cell_score + 1) as f64 / 12f64;
                            input.push(score);
                        } else {
                            input.push(0.04167f64);
                        }
                    }
                }
                organism.borrow_mut().activate(&input, &mut output);
                let fitness = *output.get(0).unwrap();
                fitness
            })));
            if game.step(minimax.get_direction()) == false {
                let score = game.get_score() as f64 / 3072f64;
                return score;
            } else {
                steps += 1;
            }
            if game.has_won() {
                return 1f64;
            }
        }
    }
}

fn main() {
    unsafe {
        local = Some(Local::now());
    }
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
    } else if target == "train" {
        #[cfg(feature = "telemetry")]
        enable_telemetry("?max_fitness=1");
        let mut population = Population::create_population(150);
        let mut environment = GameEnv;
        let mut champion: Option<Organism> = None;
        let mut max: Option<Organism> = None;
        let mut gen = 0;
        while champion.is_none() {
            population.evolve();
            population.evaluate_in(&mut environment);
            for (index, organism) in population.get_organisms().into_iter().enumerate() {
                //println!("organism fitness {}", organism.fitness);
                if organism.fitness > 2048f64 {
                    champion = Some(organism.clone());
                }
                if max.is_none() || organism.fitness > max.as_ref().unwrap().fitness {
                    max = Some(organism.clone());
                }
            }
            gen += 1;
            println!("max fitness in gen {}: {}", gen, max.as_ref().unwrap().fitness);
            let json = serde_json::to_string(&max);
            output_json(json.unwrap(), gen);
            max = None;
        }
        println!("{:?}", champion.unwrap().genome);
    }
}
