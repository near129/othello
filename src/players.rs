use crate::board::{Board, Stone};
use rand::rngs::SmallRng;
use rand::Rng;

pub trait Player {
    fn find_move(&mut self, board: &Board) -> Option<(usize, usize)>;
    fn stone(&self) -> Stone;
}
pub struct RandomPlayer {
    thred_rng: SmallRng,
    pub stone: Stone,
}
impl Player for RandomPlayer {
    fn find_move(&mut self, board: &Board) -> Option<(usize, usize)> {
        let available_squares = board.get_available_squares(self.stone);
        if available_squares.is_empty() {
            return None;
        }
        let idx = self.thred_rng.gen_range(0..available_squares.len());
        Some(available_squares[idx])
    }
    fn stone(&self) -> Stone {
        self.stone
    }
}
