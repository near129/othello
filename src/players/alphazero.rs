use std::{collections::HashMap, io::BufReader};

use super::Player;
use crate::{Board, Position, Positions, Stone, StoneCount, SIZE, UPPER_LEFT};
use anyhow::Result;
use tract_onnx::{
    prelude::*,
    tract_hir::tract_ndarray::{Array1, Array3}
};
pub struct AlphaZeroPlayer {
    pub mcts: MCTS,
}
impl AlphaZeroPlayer {
    pub fn new(model_path: &str, num_simulation: usize) -> Self {
        let model = tract_onnx::onnx()
            .model_for_path(model_path)
            .unwrap()
            .with_input_fact(
                0,
                InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 2, 8, 8)),
            )
            .unwrap()
            .into_optimized()
            .unwrap()
            .into_runnable()
            .unwrap();
        let mcts = MCTS::new(model, 1.0, num_simulation);
        AlphaZeroPlayer { mcts }
    }
}

impl Default for AlphaZeroPlayer {
    fn default() -> Self {
        let onxx_model = include_bytes!("nn_model/model.onnx");
        let model = tract_onnx::onnx()
            .model_for_read(&mut BufReader::new(&onxx_model[..]))
            .unwrap()
            .with_input_fact(
                0,
                InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 2, 8, 8)),
            )
            .unwrap()
            .into_optimized()
            .unwrap()
            .into_runnable()
            .unwrap();
        let mcts = MCTS::new(model, 1.0, 100);
        AlphaZeroPlayer { mcts }
    }
}
impl Player for AlphaZeroPlayer {
    fn find_move(&mut self, board: &Board) -> Result<Position> {
        let ret = self.mcts.search(*board)?;
        // for i in 0..8 {
        //     eprintln!("{:?}", &ret[8*i..8*i+8]);
        // }
        let idx = ret
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        // eprintln!("{} {}", idx / SIZE, idx % SIZE);
        Ok(Position(UPPER_LEFT >> idx))
    }
}

fn create_board_tensor(board: &Board) -> Tensor {
    let mut board_array = Array3::zeros((2, SIZE, SIZE));
    let (black_idx, white_idx) = if board.turn == Stone::Black {
        (0, 1)
    } else {
        (1, 0)
    };
    for i in 0..SIZE * SIZE {
        let pos = UPPER_LEFT >> i;
        if board.black & pos != 0 {
            board_array[[black_idx, i / SIZE, i % SIZE]] = 1f32;
        } else if board.white & pos != 0 {
            board_array[[white_idx, i / SIZE, i % SIZE]] = 1f32;
        }
    }
    board_array.into_tensor()
}
fn legal_move_to_array(postions: Positions) -> Array1<f32> {
    let mut arr = Array1::zeros(SIZE * SIZE);
    for i in 0..SIZE * SIZE {
        let pos = UPPER_LEFT >> i;
        if postions.0 & pos != 0 {
            arr[i] = 1f32;
        }
    }
    arr
}
type Model = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;
type State = (u64, u64);
type SA = (State, Position);
type SAHashMap<T> = HashMap<SA, T>;
type SHashMap<T> = HashMap<State, T>;
type Policy = Array1<f32>;
#[allow(clippy::upper_case_acronyms)]
#[allow(non_snake_case)]
pub struct MCTS {
    model: Model,
    cpuct: f32,
    num_simulation: usize,
    Qsa: SAHashMap<f32>,
    Nsa: SAHashMap<usize>,
    Ns: SHashMap<usize>,
    Ps: SHashMap<Policy>,
}
impl MCTS {
    pub fn new(model: Model, cpuct: f32, num_simulation: usize) -> Self {
        MCTS {
            model,
            cpuct,
            num_simulation,
            Qsa: HashMap::new(),
            Nsa: HashMap::new(),
            Ns: HashMap::new(),
            Ps: HashMap::new(),
        }
    }
    pub fn clear_cache(&mut self) {
        self.Qsa= HashMap::new();
        self.Nsa= HashMap::new();
        self.Ns =HashMap::new();
        self.Ps =HashMap::new();
    }
    pub fn search(&mut self, board: Board) -> Result<Vec<f32>> {
        for _ in 0..self.num_simulation {
            let _ = self._search(board)?;
        }
        let state = (board.black, board.white);
        let counts: Vec<usize> = (0..SIZE * SIZE)
            .map(|idx| *self.Nsa.get(&(state, Position(UPPER_LEFT >> idx))).unwrap_or(&0))
            .collect();
        let sum = *counts.iter().max().unwrap() as f32;
        Ok(counts.iter().map(|x| *x as f32 / sum).collect())
    }

    fn _search(&mut self, mut board: Board) -> Result<f32> {
        if board.finished() {
            let StoneCount { black, white } = board.count_stone();
            return Ok(-match black.cmp(&white) {
                std::cmp::Ordering::Equal => 0.,
                std::cmp::Ordering::Greater => 1.,
                std::cmp::Ordering::Less => -1.,
            });
        }
        let state = (board.black, board.white);
        Ok(if let Some(p) = self.Ps.get(&state) {
            let best = board
                .get_legal_moves()
                .to_position_list()
                .iter()
                .map(|&a| {
                    if let Some(q) = self.Qsa.get(&(state, a)) {
                        (
                            q + self.cpuct
                                * p[a.to_idx()]
                                * (*self.Ns.get(&state).unwrap() as f32).sqrt()
                                / (1. + *self.Nsa.get(&(state, a)).unwrap() as f32),
                            a,
                        )
                    } else {
                        (
                            self.cpuct
                                * p[a.to_idx()]
                                * (*self.Ns.get(&state).unwrap() as f32).sqrt(),
                            a,
                        )
                    }
                })
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                .unwrap();
            board.put(best.1)?;
            let v = self._search(board)?;
            let sa = &(state, best.1);
            if let Some(q) = self.Qsa.get_mut(sa) {
                *q = (*self.Nsa.get(sa).unwrap() as f32 + *q + v)
                    / (*self.Nsa.get(sa).unwrap() as f32 + 1.);
                *self.Nsa.get_mut(sa).unwrap() += 1;
            } else {
                self.Qsa.insert(*sa, v);
                self.Nsa.insert(*sa, 1);
            }
            *self.Ns.get_mut(&state).unwrap() += 1;
            -v
        } else {
            let input = create_board_tensor(&board).into_shape(&[1, 2, SIZE, SIZE])?;
            let output = self.model.run(tvec![input])?;
            let mut policy = output[0]
                .to_array_view::<f32>()?
                .to_shape(SIZE * SIZE)?
                .into_owned();
            let v = *output[1].to_scalar::<f32>().unwrap();
            let mask = legal_move_to_array(board.get_legal_moves());
            policy *= &mask;
            // legal_move_mask(board.get_legal_moves(), policy);
            if policy.sum() >= 0.0 {
                policy += &mask;
            }
            policy /= policy.sum();
            self.Ps.insert(state, policy);
            self.Ns.insert(state, 0);
            -v
        })
    }
}
