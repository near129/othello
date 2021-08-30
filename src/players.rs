use crate::{Board, Position};
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
enum PlayerError {
    #[error("not found legal move")]
    NotFoundLegalMove
}
pub trait Player {
    fn find_move(&mut self, board: &Board) -> Result<Position>;
}

pub mod random;
pub use random::RandomPlayer;
pub mod alphazero;
pub use alphazero::AlphaZeroPlayer;
