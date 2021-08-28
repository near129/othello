use super::Player;
use crate::{Board, Position, Stone, SIZE, UPPER_LEFT};
use rand::{rngs::SmallRng, Rng, SeedableRng};
pub struct RandomPlayer {
    thred_rng: SmallRng,
    pub stone: Stone,
}
impl RandomPlayer {
    pub fn new(stone: Stone) -> Self {
        let thred_rng = SmallRng::from_entropy();
        RandomPlayer { thred_rng, stone }
    }
}
impl Player for RandomPlayer {
    fn find_move(&mut self, board: &Board) -> Result<Position, &str> {
        let legal_moves = board.get_legal_moves();
        let n = legal_moves.count();
        if n == 0 {
            return Err("Can't put stone");
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
    fn stone(&self) -> Stone {
        self.stone
    }
}
