use std::cell::RefCell;
use std::rc::Rc;

use crate::game::{Board, Direction};

pub struct AINode {
    board: Board,
    layer: u32,
    options: Option<Vec<Rc<RefCell<AINode>>>>,
    player: Player,
}

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
                        self.add_option(AINode::new(board, self.layer + 1, Player::Min));
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
                    if board.step_rows(dir) {
                        self.add_option(AINode::new(board, self.layer + 1, Player::Max(dir)))
                    }
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

pub struct MinMaxResult {
    score: f64,
    node: Rc<RefCell<AINode>>,
}

impl MinMaxResult {
    fn new(score: f64, node: Rc<RefCell<AINode>>) -> MinMaxResult {
        MinMaxResult { score, node }
    }

    pub fn get_direction(&self) -> Direction {
        match self.node.borrow().player {
            Player::Max(dir) => dir,
            Player::Min => Direction::UP,
        }
    }

    pub fn get_score(&self) -> f64 {
        self.score
    }
}

pub trait AIScore {
    fn get_ai_score(&self) -> f64;
}

impl AIScore for Board {
    fn get_ai_score(&self) -> f64 {
        let mut max: f32 = 0.0;
        let mut cells: u32 = 0;
        let mut score: f32 = 0.0;
        for y in 0..self.height() {
            for x in 0..self.width() {
                let cell = self.get_cell(x, y);
                if cell.is_set() {
                    let cell = cell.get_score().unwrap() as f32;
                    cells += 1;
                    if cell > max {
                        max = cell;
                    }
                    if (x == 0 && y == 0)
                        || (x == 0 && y == self.height() - 1)
                        || (x == self.width() - 1 && y == 0)
                        || (x == self.width() - 1 && y == self.height() - 1)
                    {
                        score += 2i32.pow((1.25 * cell) as u32) as f32;
                    } else if x == 0 || x == self.width() - 1 || y == 0 || y == self.height() - 1 {
                        score += 2i32.pow((1.10 * cell) as u32) as f32;
                    }
                }
            }
        }
        score += max;
        score -= cells.pow(2) as f32;
        score as f64
    }
}

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
        root.build_tree(1);
        self.root = Some(Rc::new(RefCell::new(root)));
    }

    pub fn minimax(&self, heuristic: Option<Box<Fn(&Board) -> f64>>) -> MinMaxResult {
        let heuristic = Rc::new(RefCell::new(heuristic.unwrap()));
        self.minimaxfn(
            Rc::clone(self.root.as_ref().unwrap()),
            0,
            self.depth,
            None,
            None,
            Some(heuristic.clone()),
        )
    }

    pub fn minimaxfn(
        &self,
        node: Rc<RefCell<AINode>>,
        layer: u32,
        depth: u32,
        alpha: Option<f64>,
        beta: Option<f64>,
        heuristic: Option<Rc<RefCell<Box<Fn(&Board) -> f64>>>>,
    ) -> MinMaxResult {
        let mut alpha = alpha;
        let mut beta = beta;
        if node.borrow().layer < depth && node.borrow().options.is_none() {
            node.borrow_mut().add_layer();
        }
        if node.borrow().options.is_none() {
            if heuristic.is_some() {
                let score = (heuristic.unwrap().borrow())(&node.borrow().board);
                MinMaxResult::new(score, Rc::clone(&node))
            } else {
                MinMaxResult::new(node.borrow().board.get_ai_score(), Rc::clone(&node))
            }
        } else {
            match node.borrow().player {
                Player::Min => {
                    let mut max: Option<MinMaxResult> = None;
                    for child in node.borrow().options.as_ref().unwrap().iter() {
                        let value = self.minimaxfn(
                            Rc::clone(child),
                            layer + 1,
                            depth,
                            alpha,
                            beta,
                            heuristic.clone(),
                        );
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
                        let value = self.minimaxfn(
                            Rc::clone(child),
                            layer + 1,
                            depth,
                            alpha,
                            beta,
                            heuristic.clone(),
                        );
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
