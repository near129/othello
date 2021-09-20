use super::Player;
use crate::{utils::game_result, Board, Position, Stone, SIZE, UPPER_LEFT};
use anyhow::Result;
use fxhash::FxHashMap;

type State = (u64, u64);
type SA = (State, Position);
type SAHashMap<T> = FxHashMap<SA, T>;
type SHashMap<T> = FxHashMap<State, T>;

#[allow(clippy::upper_case_acronyms)]
#[allow(non_snake_case)]
pub struct MCTSPlayer {
    cpuct: f32,
    num_simulation: usize,
    N: usize,
    Qsa: SAHashMap<usize>,
    Nsa: SAHashMap<usize>,
    Ns: SHashMap<usize>,
}

impl MCTSPlayer {
    pub fn new(cpuct: f32, num_simulation: usize) -> Self {
        MCTSPlayer {
            cpuct,
            num_simulation,
            N: 1,
            Qsa: FxHashMap::default(),
            Nsa: FxHashMap::default(),
            Ns: FxHashMap::default(),
        }
    }
    pub fn clear_cache(&mut self) {
        self.Qsa = FxHashMap::default();
        self.Nsa = FxHashMap::default();
        self.Ns = FxHashMap::default();
    }
    pub fn init_search(&mut self, num_simulation: usize, board: Board) -> Result<()> {
        for _ in 0..num_simulation {
            let _ = self._search(board)?;
        }
        Ok(())
    }
    pub fn search(&mut self, board: Board) -> Result<Vec<f32>> {
        let state = if board.turn == Stone::Black {
            (board.black, board.white)
        } else {
            (board.white, board.black)
        };
        for _ in 0..self.num_simulation {
            let _ = self._search(board)?;
        }
        let counts: Vec<usize> = (0..SIZE * SIZE)
            .map(|idx| {
                *self
                    .Nsa
                    .get(&(state, Position(UPPER_LEFT >> idx)))
                    .unwrap_or(&0)
            })
            .collect();
        let sum = counts.iter().sum::<usize>() as f32;
        Ok(counts.iter().map(|x| *x as f32 / sum).collect())
    }

    fn _search(&mut self, mut board: Board) -> Result<i8> {
        let player = board.turn;
        let state = if player == Stone::Black {
            (board.black, board.white)
        } else {
            (board.white, board.black)
        };
        let best = board
            .get_legal_moves()
            .to_position_list()
            .iter()
            .map(|&a| {
                let n = *self.Ns.entry(state).or_insert(1) as f32;
                (
                    *self.Qsa.entry((state, a)).or_insert(1) as f32 / n
                        + self.cpuct * ((self.N as f32).ln() / n).sqrt(),
                    a,
                )
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();
        board.put(best.1)?;
        let v = if board.finished() {
            game_result(&board, player)
        } else if player == board.turn {
            self._search(board)?
        } else {
            -self._search(board)?
        };
        let sa = &(state, best.1);
        if v == 1 {
            *self.Qsa.get_mut(sa).unwrap() += 1;
        }
        self.N += 1;
        *self.Nsa.entry(*sa).or_default() += 1;
        *self.Ns.get_mut(&state).unwrap() += 1;
        Ok(v)
    }
}

impl Player for MCTSPlayer {
    fn find_move(&mut self, board: &Board) -> Result<Position> {
        let ret = self.search(*board)?;
        let idx = ret
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        Ok(Position(UPPER_LEFT >> idx))
    }
}
impl Default for MCTSPlayer {
    fn default() -> Self {
        MCTSPlayer::new(2f32.sqrt(), 10000)
    }
}
