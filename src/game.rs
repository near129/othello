use std::mem::swap;

use crate::{
    board::{Board, Stone, StoneCount},
    player::Player,
};

#[derive(Debug)]
pub struct TurnEndResult {
    put: Result<(), String>,
    finish: bool,
    pass: bool,
}

pub struct Game<T: Player> {
    board: Board,
    turn: Stone,
    num_turn: usize,
    finish: bool,
    player1: T,
    player2: T,
    tmp_x: Option<usize>,
    tmp_y: Option<usize>,
}

impl<T> Game<T>
where
    T: Player,
{
    #[allow(clippy::new_without_default)]
    pub fn new(mut player1: T, mut player2: T) -> Result<Game<T>, String> {
        if player1.stone() == player2.stone() {
            return Err("Must be different stone".to_string());
        }
        if player1.human() || player2.human() {
            return Err("Must be different stone".to_string());
        }
        if player1.stone() == Stone::Whilte {
            swap(&mut player1, &mut player2);
        }
        Ok(Game {
            board: Board::new(),
            turn: Stone::Black,
            num_turn: 0,
            finish: false,
            player1,
            player2,
            tmp_x: None,
            tmp_y: None
        })
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
    pub fn count_stone(&self) -> StoneCount {
        StoneCount::count(&self.board)
    }
    pub fn get_available_squares(&self) -> Vec<(usize, usize)> {
        self.board.get_available_squares(self.turn)
    }
    pub fn set_move(&mut self, x: usize, y: usize) {
        self.tmp_x = Some(x);
        self.tmp_y = Some(y);
    }
    pub fn turn_end(&mut self) -> TurnEndResult {
        if self.finish {
            return TurnEndResult {
                put: Err("Alredy game finished!".to_string()),
                finish: true,
                pass: false,
            };
        }
        let player = {
            if self.turn % 2 == 0 {
                self.player1
            } else {
                self.player2
            }
        };
        let res = if player.human() {
            if let (Some(x), Some(y)) = (self.tmp_x, self.tmp_y) {
                self.put_stone(x, y)
            } else {
                TurnEndResult {
                    put: Err("No set move".to_string()),
                    finish: false,
                    pass: false,
                }
            }
        } else {
            if let Some((x, y)) = player.find_move(&self.board) {
                self.put_stone(x, y)
            } else {
                TurnEndResult  
            }
        };
    } 
    fn put_stone(&mut self, x: usize, y: usize) -> TurnEndResult {
        if let Err(message) = self.board.put(self.turn, x, y) {
            return TurnEndResult {
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
        TurnEndResult {
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
        self.board
            .get_available_squares(self.turn.reverse())
            .is_empty()
    }
}
