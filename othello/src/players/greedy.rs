use super::Player;
use crate::{players::PlayerError, Board, Position, Stone, StoneCount, SIZE, UPPER_LEFT};
use anyhow::Result;
use rand::{rngs::SmallRng, Rng, SeedableRng};
pub struct GreedyPlayer {
    thred_rng: SmallRng,
    p: f64,
}
impl GreedyPlayer {
    pub fn new(p: f64) -> Self {
        let thred_rng = SmallRng::from_entropy();
        GreedyPlayer { thred_rng, p }
    }
}

impl Default for GreedyPlayer {
    fn default() -> Self {
        Self::new(0.8)
    }
}
impl Player for GreedyPlayer {
    fn find_move(&mut self, board: &Board) -> Result<Position> {
        let legal_moves = board.get_legal_moves();
        let n = legal_moves.count();
        if n == 0 {
            return Err(PlayerError::NotFoundLegalMove.into());
        }
        if self.thred_rng.gen_bool(self.p) {
            let mut best = None;
            let mut best_num = 0;
            let turn = board.turn;
            for p in legal_moves.to_position_list() {
                let mut tmp = *board;
                tmp.put(p)?;
                let StoneCount { black, white } = tmp.count_stone();
                let num = if turn == Stone::Black { black } else { white };
                if best_num < num {
                    best_num = num;
                    best = Some(p);
                }
            }
            return Ok(best.unwrap());
        } else {
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
        }
        unreachable!()
    }
}
