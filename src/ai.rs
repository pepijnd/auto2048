use std::collections::HashMap;

use std::cell::RefCell;
use std::rc::Rc;

use crate::game::{Board, Direction};

#[derive(Debug)]
pub struct AINode {
    board: Board,
    layer: u32,
    options: Option<Vec<Rc<RefCell<AINode>>>>,
    player: Player,
}

#[derive(Debug)]
enum Player {
    Max(Direction),
    Min,
}

impl AINode {
    fn new(board: Board, layer: u32, player: Player) -> AINode {
        AINode {
            board,
            layer,
            options: None,
            player,
        }
    }

    fn add_option(&mut self, node: AINode) {
        if self.options.is_none() {
            self.options = Some(Vec::new());
        }
        self.options
            .as_mut()
            .unwrap()
            .push(Rc::new(RefCell::new(node)));
    }

    fn add_layer(&mut self) {
        match self.player {
            Player::Max(dir) => {
                let max = match dir {
                    Direction::LEFT | Direction::RIGHT => self.board.height(),
                    Direction::DOWN | Direction::UP => self.board.width(),
                };
                for i in 0..max {
                    let mut board = self.board.clone();
                    let ok = board.step_add_index(dir, i);
                    if ok {
                        let new_node = AINode::new(board, self.layer + 1, Player::Min);
                        self.add_option(new_node);
                    }
                }
            }
            Player::Min => {
                let dirs = vec![
                    Direction::DOWN,
                    Direction::LEFT,
                    Direction::RIGHT,
                    Direction::UP,
                ];
                for dir in dirs {
                    let mut board = self.board.clone();
                    board.step_rows(dir);
                    self.add_option(AINode::new(board, self.layer + 1, Player::Max(dir)))
                }
            }
        }
    }

    fn build_sub_tree(&mut self, max_depth: u32) {
        if self.options.is_some() {
            for option in self.options.as_mut().unwrap() {
                option.borrow_mut().build_tree(max_depth);
            }
        }
    }

    fn build_tree(&mut self, max_depth: u32) {
        self.add_layer();
        if self.layer < max_depth {
            self.build_sub_tree(max_depth);
        }
    }
}

#[derive(Debug)]
pub struct MinMaxResult {
    score: i32,
    node: Rc<RefCell<AINode>>,
}

impl MinMaxResult {
    fn new(score: i32, node: Rc<RefCell<AINode>>) -> MinMaxResult {
        MinMaxResult { score, node }
    }

    pub fn get_direction(&self) -> Direction {
        match self.node.borrow().player {
            Player::Max(dir) => dir,
            Player::Min => Direction::UP,
        }
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }
}

pub trait AIScore {
    fn get_ai_score(&self) -> i32;
}

impl AIScore for Board {
    fn get_ai_score(&self) -> i32 {
        let mut max: f32 = 0.0;
        let mut cells: u32 = 0;
        let mut score: f32 = 0.0;
        for y in 0..self.height() {
            for x in 0..self.width() {
                let cell = self.get_cell(x, y);

                if cell.is_set() {
                    cells += 1;
                    let mut mult = 1.0;
                    if (x == 0 && y == 0)
                        || (x == 0 && y == self.height() - 1)
                        || (x == self.width() - 1 && y == 0)
                        || (x == self.width() - 1 && y == self.height() - 1)
                    {
                        mult = 1.25;
                    } else if x == 0 || x == self.width() - 1 || y == 0 || y == self.height() - 1 {
                        mult = 1.15;
                    } else {
                        mult = 0.95;
                    }
                    if mult * cell.get_score().unwrap() as f32 > max {
                        max = mult * cell.get_score().unwrap() as f32;
                    }
                    score += mult * 2i32.pow(cell.get_score().unwrap()) as f32;
                }
            }
        }

        let score = score as i32 - cells as i32;
        score
    }
}

#[derive(Debug)]
pub struct AI {
    board: Board,
    depth: u32,
    root: Option<Rc<RefCell<AINode>>>,
}

impl AI {
    pub fn new(board: &Board, depth: u32) -> AI {
        AI {
            board: board.clone(),
            depth,
            root: None,
        }
    }

    pub fn build_tree(&mut self) {
        let mut root = AINode::new(self.board.clone(), 0, Player::Min);
        root.build_tree(self.depth);
        self.root = Some(Rc::new(RefCell::new(root)));
    }

    pub fn minimax(&self) -> MinMaxResult {
        Self::minimaxfn(
            Rc::clone(self.root.as_ref().unwrap()),
            self.depth,
            None,
            None,
        )
    }

    pub fn minimaxfn(
        node: Rc<RefCell<AINode>>,
        depth: u32,
        alpha: Option<i32>,
        beta: Option<i32>,
    ) -> MinMaxResult {
        let mut alpha = alpha;
        let mut beta = beta;
        if node.borrow().options.is_none() {
            MinMaxResult::new(node.borrow().board.get_ai_score(), Rc::clone(&node))
        } else {
            match node.borrow().player {
                Player::Min => {
                    let mut max: Option<MinMaxResult> = None;
                    for child in node.borrow().options.as_ref().unwrap().iter() {
                        let mut value = Self::minimaxfn(Rc::clone(child), depth + 1, alpha, beta);
                        if max.is_none() || value.score > max.as_ref().unwrap().score {
                            max = Some(MinMaxResult::new(value.score, Rc::clone(child)));
                        }
                        if alpha.is_none() || value.score > alpha.unwrap() {
                            alpha = Some(value.score);
                        }
                        if alpha.is_some() && beta.is_some() && alpha.unwrap() > beta.unwrap() {
                            break;
                        }
                    }
                    max.unwrap()
                }
                Player::Max(_) => {
                    let mut max: Option<MinMaxResult> = None;
                    for child in node.borrow().options.as_ref().unwrap().iter() {
                        let mut value = Self::minimaxfn(Rc::clone(child), depth + 1, alpha, beta);
                        if max.is_none() || value.score < max.as_ref().unwrap().score {
                            max = Some(MinMaxResult::new(value.score, Rc::clone(child)));
                        }
                        if beta.is_none() || value.score < beta.unwrap() {
                            beta = Some(value.score);
                        }
                        if alpha.is_some() && beta.is_some() && alpha.unwrap() > beta.unwrap() {
                            break;
                        }
                    }
                    max.unwrap()
                }
            }
        }
    }
}
