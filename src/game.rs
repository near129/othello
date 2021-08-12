use crate::board::{Board, Stone, StoneCounter};

#[derive(Debug)]
pub struct PutStoneResult {
    put: Result<(), String>,
    finish: bool,
    pass: bool,
}

pub struct Game {
    board: Board,
    turn: Stone,
    finish: bool,
}

impl Game {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Game {
        Game {
            board: Board::new(),
            turn: Stone::Black,
            finish: false,
        }
    }
    pub fn turn(&self) -> &Stone {
        &self.turn
    }
    pub fn finish(&self) -> bool {
        self.finish
    }
    pub fn board(&self) -> &Board {
        &self.board
    }
    pub fn count_stone(&self) -> StoneCounter {
        StoneCounter::count(&self.board)
    }
    pub fn get_available_squares(&self) -> Vec<(usize, usize)> {
        self.board.get_available_squares(self.turn)
    }
    pub fn put_stone(&mut self, x: usize, y: usize) -> PutStoneResult {
    // pub fn put_stone(&mut self, stone: Stone x: usize, y: usize) -> PutStoneResult {
    // 明示的に自分のターンを指定させる
        if self.finish {
            return PutStoneResult {
                put: Err("Alredy game finished!".to_string()),
                finish: true,
                pass: false,
            };
        }
        if let Err(message) = self.board.put(self.turn, x, y) {
            return PutStoneResult {
                put: Err(message),
                finish: self.finish,
                pass: false,
            };
        }
        self.finish = self.is_finish();
        let pass = self.is_pass();
        if !(self.finish || pass) {
            self.turn = self.turn.reverse();
        }
        PutStoneResult {
            put: Ok(()),
            finish: self.finish,
            pass,
        }
    }
    fn is_finish(&self) -> bool {
        self.board.get_available_squares(Stone::Black).is_empty()
            && self.board.get_available_squares(Stone::Whilte).is_empty()
    }
    fn is_pass(&self) -> bool {
        self.board.get_available_squares(self.turn.reverse()).is_empty()
    }
}
