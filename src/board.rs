use std::fmt::{self, Debug, Display};

use crate::othello_logic::{legal_move, put};
use anyhow::Result;
use thiserror::Error;

pub const SIZE: usize = 8;
pub const UPPER_LEFT: u64 = 0x8000000000000000;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stone {
    Whilte,
    Black,
}
impl Stone {
    pub fn reverse(&self) -> Stone {
        if *self == Stone::Black {
            Stone::Whilte
        } else {
            Stone::Black
        }
    }
}
pub struct StoneCount {
    pub black: usize,
    pub white: usize,
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Position(pub u64);
impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Position(UPPER_LEFT >> (y * SIZE + x))
    }
    pub fn to_idx(&self) -> usize {
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if self.0 & pos != 0 {
                return i;
            }
        }
        unreachable!()
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if self.0 & pos != 0 {
                return writeln!(f, "({}, {})", i / SIZE, i % SIZE);
            }
        }
        unreachable!()
    }
}
impl From<(usize, usize)> for Position {
    fn from(xy: (usize, usize)) -> Position {
        // for i in 0..SIZE * SIZE {
        //     let pos = UPPER_LEFT >> i;
        //     if p.0 & pos != 0 {
        //         return (i / SIZE, i % SIZE);
        //     }
        // }
        // unreachable!()
        Position(UPPER_LEFT >> (xy.1 * SIZE + xy.0))
    }
}
pub struct Positions(pub u64);
impl Positions {
    pub fn count(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn to_map(&self) -> [[bool; SIZE]; SIZE] {
        let mut m = [[false; SIZE]; SIZE];
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if self.0 & pos != 0 {
                m[i / SIZE][i % SIZE] = true;
            }
        }
        m
    }
    pub fn to_position_list(&self) -> Vec<Position> {
        let mut v = vec![];
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if self.0 & pos != 0 {
                v.push(Position(pos));
            }
        }
        v
    }
}
impl From<Positions> for Vec<(usize, usize)> {
    fn from(p: Positions) -> Self {
        let mut ps = vec![];
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if p.0 & pos != 0 {
                ps.push((i / SIZE, i % SIZE));
            }
        }
        ps
    }
}
#[derive(Clone, Copy)]
pub struct Board {
    pub turn: Stone,
    pub black: u64,
    pub white: u64,
}
#[derive(Debug, Error)]
enum BoardError {
    #[error("illegal position {0}")]
    IllegalPositionError(Position),
    #[error("the game is alredy over")]
    AlredyGameOverError,
}
type BoardArray = [[Option<Stone>; SIZE]; SIZE];

const BLACK_STONE_STRING: &str = "⚪️";
const WHITE_STONE_STRING: &str = "⚫️";

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = "  a b c d e f g h\n".to_string();
        for i in 0..SIZE * SIZE {
            if i % SIZE == 0 {
                s.push(std::char::from_digit((i / SIZE) as u32, 10).unwrap());
            }
            let pos = UPPER_LEFT >> i;
            let stone = match (self.black & pos != 0, self.white & pos != 0) {
                (true, false) => BLACK_STONE_STRING,
                (false, true) => WHITE_STONE_STRING,
                (false, false) => "・",
                _ => unreachable!(),
            };
            s.push_str(stone);
            if i % SIZE == SIZE - 1 {
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}
impl From<Board> for BoardArray {
    fn from(board: Board) -> Self {
        let mut board_array = [[None; SIZE]; SIZE];
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if board.black & pos != 0 {
                board_array[i / SIZE][i % SIZE] = Some(Stone::Black);
            } else if board.white & pos != 0 {
                board_array[i / SIZE][i % SIZE] = Some(Stone::Whilte);
            }
        }
        board_array
    }
}
impl Board {
    pub fn new() -> Self {
        Board {
            turn: Stone::Black,
            black: 0x0000000810000000,
            white: 0x0000001008000000,
        }
    }
    pub fn init(&mut self) {
        self.black = 0x0000000810000000;
        self.white = 0x0000001008000000;
    }
    pub fn get_legal_moves(&self) -> Positions {
        let ret = if self.turn == Stone::Black {
            legal_move(self.black, self.white)
        } else {
            legal_move(self.white, self.black)
        };
        Positions(ret)
    }
    pub fn finished(&self) -> bool {
        self.get_legal_moves().0 == 0
    }
    pub fn put(&mut self, pos: Position) -> Result<()> {
        if pos.0 & self.get_legal_moves().0 == 0 {
            return Err(BoardError::IllegalPositionError(pos).into());
        }
        if self.finished() {
            return Err(BoardError::AlredyGameOverError.into());
        }
        if self.turn == Stone::Black {
            let (black, white) = put(self.black, self.white, pos.0);
            self.black = black;
            self.white = white;
            self.turn = if legal_move(self.white, self.black) == 0 {
                Stone::Black
            } else {
                Stone::Whilte
            };
        } else {
            let (white, black) = put(self.white, self.black, pos.0);
            self.black = black;
            self.white = white;
            self.turn = if legal_move(self.black, self.white) == 0 {
                Stone::Whilte
            } else {
                Stone::Black
            };
        }
        Ok(())
    }
    pub fn count_stone(&self) -> StoneCount {
        StoneCount {
            black: self.black.count_ones() as usize,
            white: self.white.count_ones() as usize,
        }
    }
    pub fn create_board_array(&self) -> BoardArray {
        let mut board_array = [[None; SIZE]; SIZE];
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if self.black & pos != 0 {
                board_array[i / SIZE][i % SIZE] = Some(Stone::Black);
            } else if self.white & pos != 0 {
                board_array[i / SIZE][i % SIZE] = Some(Stone::Whilte);
            }
        }
        board_array
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
