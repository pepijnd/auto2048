use rsrl::{
    domains::{Domain, Observation, Transition},
    geometry::{continuous::Interval, discrete::Ordinal, product::LinearSpace, Surjection, Vector},
};

use crate::game::{Board, Cell, Direction, Game};

const REWARD_STEP: f64 = -1.0;
const REWARD_GOAL: f64 = 0.0;

pub struct GameDomain {
    game: Game,
}

impl From<usize> for Direction {
    fn from(dir: usize) -> Direction {
        match dir {
            0 => Direction::UP,
            1 => Direction::RIGHT,
            2 => Direction::DOWN,
            3 => Direction::LEFT,

            _ => Direction::DOWN,
        }
    }
}

trait GameState {
    fn get_state(&self) -> Vec<f64>;
}

impl GameState for Game {
    fn get_state(&self) -> Vec<f64> {
        let mut s = Vec::new();
        for cell in self.get_board().board_data().iter() {
            if cell.is_set() {
                s.push(cell.get_score().unwrap() as f64);
            } else {
                s.push(0f64);
            }
        }
        s
    }
}

impl Default for GameDomain {
    fn default() -> GameDomain {
        GameDomain { game: Game::new() }
    }
}

impl GameDomain {
    pub fn get_score(&self) -> i32 {
        self.game.get_score()
    }
}

impl Domain for GameDomain {
    type StateSpace = LinearSpace<Interval>;
    type ActionSpace = Ordinal;

    fn emit(&self) -> Observation<Vector<f64>> {
        let s = Vector::from_vec(self.game.get_state());

        if self.is_terminal() {
            Observation::Terminal(s)
        } else {
            Observation::Full(s)
        }
    }

    fn step(&mut self, action: usize) -> Transition<Vector<f64>, usize> {
        let from = self.emit();

        self.game.step(Direction::from(action));
        let to = self.emit();
        let reward = self.reward(&from, &to);

        Transition {
            from,
            action,
            reward,
            to,
        }
    }

    fn is_terminal(&self) -> bool {
        self.game.is_over()
    }

    fn reward(&self, from: &Observation<Vector<f64>>, to: &Observation<Vector<f64>>) -> f64 {
        let from = match from {
            Observation::Full(s) | Observation::Terminal(s) => {
                let mut c = 0;
                for cell in s.iter() {
                    if *cell < 1f64 {
                        c += 1;
                    }
                }
                c
            }
            _ => 0,
        };
        let to = match to {
            Observation::Full(s) | Observation::Terminal(s) => {
                let mut c = 0;
                for cell in s.iter() {
                    if *cell < 1f64 {
                        c += 1;
                    }
                }
                c
            }
            _ => 0,
        };
        (from - to) as f64 + 1.0
    }

    fn state_space(&self) -> Self::StateSpace {
        let mut s = LinearSpace::empty();
        for _ in 0..16 {
            s = s + Interval::bounded(0f64, 11f64);
        }
        s
    }

    fn action_space(&self) -> Self::ActionSpace {
        Ordinal::new(4)
    }
}
