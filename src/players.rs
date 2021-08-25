use crate::board::{Board, SIZE, Stone};
use rand::{Rng, SeedableRng, rngs::SmallRng};

pub trait Player {
    fn find_move(&mut self, board: &Board) -> Result<(usize, usize), String>;
    fn stone(&self) -> Stone;
}
pub struct RandomPlayer {
    thred_rng: SmallRng,
    pub stone: Stone,
}
impl RandomPlayer {
    pub fn new(stone: Stone) -> Self {
        let thred_rng = SmallRng::from_entropy();
        RandomPlayer {
            thred_rng, stone
        }
    }
}
impl Player for RandomPlayer {
    fn find_move(&mut self, board: &Board) -> Result<(usize, usize), String> {
        let available_squares = board.get_available_squares(self.stone);
        let n = available_squares.iter().flatten().filter(|b| **b).count();
        if n == 0 {
            return Err("Can't put stone".to_string());
        }
        let mut idx = self.thred_rng.gen_range(0..n);
        for i in 0..SIZE {
            for j in 0..SIZE {
                if available_squares[j][i] {
                    if idx == 0 {
                        return Ok((i, j));
                    } else {
                        idx -= 1;
                    }
                }
            }
        }
        unreachable!()
    }
    fn stone(&self) -> Stone {
        self.stone
    }
}
