use super::Player;
use crate::{players::PlayerError, Board, Position, SIZE, UPPER_LEFT};
use anyhow::Result;
use rand::{rngs::SmallRng, Rng, SeedableRng};
pub struct RandomPlayer {
    thred_rng: SmallRng,
}
impl RandomPlayer {
    pub fn new() -> Self {
        let thred_rng = SmallRng::from_entropy();
        RandomPlayer { thred_rng }
    }
}

impl Default for RandomPlayer {
    fn default() -> Self {
        Self::new()
    }
}
impl Player for RandomPlayer {
    fn find_move(&mut self, board: &Board) -> Result<Position> {
        let legal_moves = board.get_legal_moves();
        let n = legal_moves.count();
        if n == 0 {
            return Err(PlayerError::NotFoundLegalMove.into());
        }
        let mut idx = self.thred_rng.gen_range(0..n);
        for i in 0..SIZE * SIZE {
            let pos = UPPER_LEFT >> i;
            if legal_moves.0 & pos != 0 {
                if idx == 0 {
                    return Ok(Position(pos));
                }
                idx -= 1;
            }
        }
        unreachable!()
    }
}
