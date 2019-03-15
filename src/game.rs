use rand::{prelude::SliceRandom, thread_rng, Rng};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug)]
pub struct Game {
    scoreTarget: u32,
    moves: u32,
    board: Board,
}

impl Game {
    pub fn new() -> Game {
        Game {
            scoreTarget: 11,
            moves: 0,
            board: Board::new(),
        }
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn get_mut_board(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn step(&mut self, dir: Direction) -> bool {
        self.board.step(dir)
    }

    pub fn print_board(&self) {
        self.board.print_board();
    }

    pub fn get_score(&self) -> i32 {
        let mut cells: i32 = 0;
        let mut score: i32 = 0;

        for cell in self.board.data.iter() {
            if cell.is_set() {
                cells += 1;
                score += 2_i32.pow(cell.get_score().unwrap());
            }
        }

        score - cells + 1
    }

    pub fn has_won(&self) -> bool {
        for cell in self.board.data.iter() {
            if cell.is_set() && cell.get_score().unwrap() == self.scoreTarget {
                return true;
            }
        }
        false
    }

    pub fn reset(&mut self) {
        self.board = Board::new();
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    height: usize,
    width: usize,
    data: Vec<Cell>,
}

impl Board {
    fn new() -> Board {
        let data = vec![Cell::new(); 4 * 4];
        Board {
            height: 4,
            width: 4,
            data,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        self.data.get(y * self.width + x).unwrap()
    }

    pub fn get_mut_cell(&mut self, x: usize, y: usize) -> &mut Cell {
        self.data.get_mut(y * self.width + x).unwrap()
    }

    pub fn get_row(&self, index: usize) -> Box<[&Cell]> {
        let mut row = Vec::with_capacity(self.width);
        for i in 0..self.width {
            row.push(self.get_cell(i, index));
        }
        row.into_boxed_slice()
    }

    pub fn get_col(&self, index: usize) -> Box<[&Cell]> {
        let mut col = Vec::with_capacity(self.height);
        for i in 0..self.height {
            col.push(self.get_cell(index, i));
        }
        col.into_boxed_slice()
    }

    pub fn get_mut_row(&mut self, index: usize) -> Box<[&mut Cell]> {
        let mut row = Vec::with_capacity(self.width);
        for (i, cell) in self.data.iter_mut().enumerate() {
            if i / self.width == index {
                row.push(cell);
            }
        }
        row.into_boxed_slice()
    }

    pub fn get_mut_col(&mut self, index: usize) -> Box<[&mut Cell]> {
        let mut col = Vec::with_capacity(self.width);
        for (i, cell) in self.data.iter_mut().enumerate() {
            if i % self.width == index {
                col.push(cell);
            }
        }
        col.into_boxed_slice()
    }

    pub fn step(&mut self, dir: Direction) -> bool {
        self.step_rows(dir);
        self.step_add(dir)
    }

    pub fn step_rows(&mut self, dir: Direction) -> bool {
        match dir {
            Direction::UP | Direction::DOWN => self.step_vertical(dir),
            Direction::LEFT | Direction::RIGHT => self.step_horizontal(dir),
        }
    }

    pub fn step_add(&mut self, dir: Direction) -> bool {
        match dir {
            Direction::UP => self.get_mut_row(self.height - 1).add_random_zero(),
            Direction::DOWN => self.get_mut_row(0).add_random_zero(),
            Direction::LEFT => self.get_mut_col(self.width - 1).add_random_zero(),
            Direction::RIGHT => self.get_mut_col(0).add_random_zero(),
        }
    }

    pub fn step_add_index(&mut self, dir: Direction, index: usize) -> bool {
        let row = match dir {
            Direction::UP => self.get_mut_row(self.height - 1),
            Direction::DOWN => self.get_mut_row(0),
            Direction::LEFT => self.get_mut_col(self.width - 1),
            Direction::RIGHT => self.get_mut_col(0),
        };
        if row[index].is_set() {
            false
        } else {
            row[index].set_score(0);
            true
        }
    }

    fn step_vertical(&mut self, dir: Direction) -> bool {
        let mut ok = false;
        for i in 0..self.width {
            let mut col = self.get_mut_col(i);
            if let Direction::DOWN = dir {
                col.reverse();
            }
            if col.combine_row() {
                ok = true;
            }
        }
        ok
    }

    fn step_horizontal(&mut self, dir: Direction) -> bool {
        let mut ok = false;
        for i in 0..self.height {
            let mut row = self.get_mut_row(i);
            if let Direction::RIGHT = dir {
                row.reverse();
            }
            if row.combine_row() {
                ok = true;
            }
        }
        ok
    }

    pub fn board_data(&self) -> &Vec<Cell> {
        &self.data
    }

    pub fn print_board(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!("{}", self.get_cell(x, y).as_symbol());
            }
            print!("\n");
        }
    }
}

pub trait ModRow {
    fn add_random_zero(self) -> bool;
    fn combine_row(&mut self) -> bool;
}

impl ModRow for Box<[&mut Cell]> {
    fn add_random_zero(mut self) -> bool {
        let mut rng = thread_rng();
        self.shuffle(&mut rng);

        for n in self.iter_mut() {
            if !n.is_set() {
                n.set_score(0);
                return true;
            }
        }
        false
    }

    fn combine_row(&mut self) -> bool {
        let mut ok = false;
        for index in 0..self.len() {
            for i in index + 1..self.len() {
                if self[index].is_set() {
                    if self[index].get_score() == self[i].get_score() {
                        self[index].incr_score();
                        self[i].set_none();
                        ok = true;
                        break;
                    } else if self[i].is_set() {
                        break;
                    }
                } else if self[i].is_set() {
                    self[index].set_score(self[i].get_score().unwrap());
                    self[i].set_none();
                    ok = true;
                }
            }
        }
        ok
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    score: Option<u32>,
}

impl Cell {
    fn new() -> Cell {
        Cell { score: None }
    }

    pub fn is_set(&self) -> bool {
        self.score.is_some()
    }

    pub fn get_score(&self) -> Option<u32> {
        self.score
    }

    pub fn set_score(&mut self, score: u32) {
        self.score = Some(score);
    }

    pub fn set_none(&mut self) {
        self.score = None;
    }

    pub fn incr_score(&mut self) -> bool {
        if !self.is_set() {
            false
        } else {
            self.score = Some(self.score.unwrap() + 1);
            true
        }
    }

    pub fn as_symbol(&self) -> String {
        if self.is_set() {
            return self.score.unwrap().to_string();
        } else {
            return String::from("*");
        }
    }

    pub fn as_string(&self) -> String {
        if self.is_set() {
            return 2_i32.pow(self.score.unwrap()).to_string();
        } else {
            return String::from("");
        }
    }
}
