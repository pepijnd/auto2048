use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

use std::time::Duration;

use crate::ai::AIScore;
use crate::ai::AI;
use crate::game::{Direction, Game};

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }

    pub fn run_app(&self) -> Result<(), String> {
        let mut game = Game::new();

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

        let width = 400;
        let height = 600;

        let window = video_subsystem
            .window("auto2048", width, height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        let mut font = ttf_context.load_font("example.ttf", 128)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        canvas.set_draw_color(Color::RGB(240, 240, 240));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump()?;

        let mut auto_run = false;
        let mut frame = 0;
        let mut avg = false;

        let mut starts = Vec::new();

        'running: loop {
            frame += 1;

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {game.step(Direction::LEFT);},
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {game.step(Direction::UP);},
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {game.step(Direction::DOWN);},
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {game.step(Direction::RIGHT);},
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => {
                        auto_run = !auto_run;
                        if auto_run == false {
                            avg = true;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => game.reset(),
                    _ => {}
                }
            }
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));


            use std::time::Instant;
            if auto_run && frame != 0 {
                let mut ai = AI::new(&game.get_board(), 9);
                ai.build_tree();
                let start = Instant::now();
                let minimax = ai.minimax();

                let start = start.elapsed();

                starts.push(start);

                println!(
                    "{}\t{}\t{}\t{:?}",
                    match minimax.get_direction() {
                        Direction::UP => "Up",
                        Direction::DOWN => "Down",
                        Direction::LEFT => "Left",
                        Direction::RIGHT => "Right",
                    },
                    minimax.get_score(),
                    game.get_board().get_ai_score(),
                    start
                );

                if game.step(minimax.get_direction()) == false {
                    auto_run = false;
                }
            }

            if avg {
                let mut start_time = Duration::new(0, 0);
                for start in starts.iter() {
                    start_time += *start;
                }

                println!("avg\tstart:{:?}", start_time / starts.len() as u32);

                starts.clear();

                avg = false;
            }

            canvas.set_draw_color(Color::RGB(240, 240, 240));
            canvas.clear();

            let board = game.get_board();

            let rw = 380 / board.width() as i32;
            let rh = 380 / board.height() as i32;

            for x in 0..board.width() {
                for y in 0..board.height() {
                    let rx = x as i32 * rw;
                    let ry = y as i32 * rh;
                    let rect = Rect::new(rx + 10, ry + 10, rw as u32, rh as u32);
                    canvas.set_draw_color(Color::RGB(255, 225, 225));
                    canvas.fill_rect(rect)?;
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas.draw_rect(rect)?;

                    if board.get_cell(x, y).is_set() {
                        let surface = font
                            .render(&board.get_cell(x, y).as_string())
                            .blended(Color::RGBA(0, 0, 0, 255))
                            .map_err(|e| e.to_string())?;
                        let texture = texture_creator
                            .create_texture_from_surface(&surface)
                            .map_err(|e| e.to_string())?;
                        let TextureQuery { width, height, .. } = texture.query();
                        canvas.copy(&texture, None, Some(rect))?;
                    }
                }
            }

            let score = game.get_score();

            let surface = font
                .render(&format!("Score: {}", score))
                .blended(Color::RGBA(0, 0, 0, 255))
                .map_err(|e| e.to_string())?;
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            let TextureQuery { width, height, .. } = texture.query();
            canvas.copy(&texture, None, Some(Rect::new(150, 400, 100, 40)))?;

            canvas.present();
        }

        Ok(())
    }
}
