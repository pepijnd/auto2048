use std::collections::HashMap;

use crate::game::{Board, Direction};
use std::cmp;

pub struct AINode {
    board: Board,
    layer: u32,
    score: Option<i32>,
    options: Option<HashMap<Direction, Vec<AINode>>>,
}

impl AINode {
    fn new(board: Board, layer: u32) -> AINode {
        AINode {
            board,
            layer,
            options: None,
            score: None,
        }
    }

    fn set_score(&mut self) {
        self.score = Some(self.board.get_ai_score())
    }

    fn add_option(&mut self, dir: Direction, node: AINode) {
        if self.options.is_none() {
            self.options = Some(HashMap::new());
        }
        if !self.options.as_ref().unwrap().contains_key(&dir) {
            self.options.as_mut().unwrap().insert(dir, Vec::new());
        }
        self.options
            .as_mut()
            .unwrap()
            .get_mut(&dir)
            .unwrap()
            .push(node);
    }

    fn add_layer(&mut self) {
        let dirs = vec![
            Direction::DOWN,
            Direction::LEFT,
            Direction::RIGHT,
            Direction::UP,
        ];
        for dir in dirs.iter() {
            let max = match dir {
                Direction::LEFT | Direction::RIGHT => self.board.height(),
                Direction::DOWN | Direction::UP => self.board.width(),
            };
            for i in 0..max {
                let mut board = self.board.clone();
                board.step_rows(*dir);
                let ok = board.step_add_index(*dir, i);
                if ok {
                    let new_node = AINode::new(board, self.layer + 1);
                    self.add_option(*dir, new_node);
                }
            }
        }
    }

    fn get_highest(&self) -> &AINode {
        if self.options.is_none() {
            &self
        } else {
            let mut max_score = None;
            let mut max_option = None;

            for dir in self.options.as_ref().unwrap().values() {
                for option in dir.iter() {
                    let score = option.board.get_ai_score();
                    if max_score.is_none() || score > max_score.unwrap() {
                        max_score = Some(score);
                        max_option = Some(option);
                    }
                }
            }

            &max_option.unwrap()
        }
    }

    fn build_sub_tree(&mut self, max_depth: u32) {
        if self.options.is_some() {
            for dir in self.options.as_mut().unwrap().values_mut() {
                for option in dir.iter_mut() {
                    option.build_tree(max_depth);
                }
            }
        }
    }

    fn build_tree(&mut self, max_depth: u32) {
        self.add_layer();
        if self.layer < max_depth {
            self.build_sub_tree(max_depth);
        }
    }

    fn minimax(&self, depth: u32, alpha: Option<i32>, beta: Option<i32>) -> MinMaxResult {
        let mut alpha = alpha;
        let mut beta = beta;
        if self.options.is_none() {
            MinMaxResult::score(self.board.get_ai_score(), self)
        } else {
            let dirs = vec![
                Direction::DOWN,
                Direction::LEFT,
                Direction::RIGHT,
                Direction::UP,
            ];
            let mut values: HashMap<Direction, MinMaxResult> = HashMap::new();
            for dir in dirs.iter() {
                if self.options.as_ref().unwrap().get(dir).is_some() {
                    for option in self.options.as_ref().unwrap().get(dir).unwrap().iter() {
                        let mut result = option.minimax(depth - 1, alpha, beta);
                        if values.get(dir).is_none()
                            || result.score < values.get(dir).unwrap().score
                        {
                            result.direction = Some(*dir);
                            values.insert(*dir, result);
                        }
                        if alpha.is_none() || values.get(dir).unwrap().score < alpha.unwrap() {
                            alpha = Some(values.get(dir).unwrap().score);
                        }
                        if beta.is_some() && alpha.is_some() && alpha.unwrap() > beta.unwrap() {
                            break;
                        }
                    }
                }
            }
            let mut max: Option<MinMaxResult> = None;
            for (_dir, result) in values.drain() {
                if max.is_none() || result.score > max.as_ref().unwrap().score {
                    max = Some(result);
                }
                if alpha.is_none() || max.as_ref().unwrap().score > alpha.unwrap() {
                    alpha = Some(max.as_ref().unwrap().score);
                }
                if beta.is_some() && alpha.is_some() && alpha.unwrap() > beta.unwrap() {
                    break;
                }
            }
            max.unwrap()
        }
    }
}

pub struct MinMaxResult<'a> {
    score: i32,
    node: &'a AINode,
    direction: Option<Direction>,
}

impl<'a> MinMaxResult<'a> {
    fn new(score: i32, node: &'a AINode, direction: Direction) -> MinMaxResult<'a> {
        MinMaxResult {
            score,
            node,
            direction: Some(direction),
        }
    }

    fn score(score: i32, node: &'a AINode) -> MinMaxResult<'a> {
        MinMaxResult {
            score,
            node,
            direction: None,
        }
    }

    pub fn get_direction(&self) -> Direction {
        self.direction.unwrap_or_else(|| Direction::DOWN)
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
                    if (x==0 && y==0) || (x==0 && y==self.height()-1) || (x==self.width()-1 && y==0) || (x==self.width()-1 && y==self.height()-1) {
                        mult = 1.25;
                    } else if x==0 || x==self.width()-1 || y==0 || y==self.height()-1 {
                        mult = 1.15;
                    } else {
                        mult = 0.95;
                    }
                    if mult*cell.get_score().unwrap() as f32 > max {
                        max = mult*cell.get_score().unwrap() as f32;
                    }
                    score += mult*2i32.pow(cell.get_score().unwrap()) as f32;
                }
                

            }
        }

        let score = score as i32 - cells as i32;
        score
    }
}

pub struct AI {
    board: Board,
    root: Option<AINode>,
    depth: Option<u32>,
}

impl<'a> AI {
    pub fn new(board: &Board) -> AI {
        AI {
            board: board.clone(),
            root: None,
            depth: None,
        }
    }

    pub fn build_tree(&mut self, depth: u32) {
        let mut root = AINode::new(self.board.clone(), 0);
        root.build_tree(depth);
        self.root = Some(root);
        self.depth = Some(depth);
    }

    pub fn minimax(&'a self) -> MinMaxResult<'a> {
        self.root
            .as_ref()
            .unwrap()
            .minimax(self.depth.unwrap(), None, None)
    }
}
