use crate::{Board, Position, Stone};

pub trait Player {
    fn find_move(&mut self, board: &Board) -> Result<Position, &str>;
    fn stone(&self) -> Stone;
}

pub mod random;
pub use random::RandomPlayer;
pub mod alphazero;
