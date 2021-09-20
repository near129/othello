use std::io::BufReader;

use super::Player;
use crate::{Board, Position, Positions, SIZE, Stone, UPPER_LEFT, utils::{create_board_tensor, game_result}};
use anyhow::Result;
use fxhash::FxHashMap;
use rand::prelude::*;
use rand_distr::Dirichlet;
use tract_onnx::{prelude::*, tract_hir::tract_ndarray::Array1};
pub struct AlphaZeroPlayer {
    pub mcts: MCTS,
}
impl AlphaZeroPlayer {
    pub fn new(num_simulation: usize) -> Self {
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
        let mcts = MCTS::new(model, 1.0, num_simulation);
        AlphaZeroPlayer { mcts }
    }
    pub fn new_from_model_path(model_path: &str, num_simulation: usize) -> Self {
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
        let mcts = MCTS::new(model, 1.0, 5000);
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
type SAHashMap<T> = FxHashMap<SA, T>;
type SHashMap<T> = FxHashMap<State, T>;
type Policy = Array1<f32>;
#[allow(clippy::upper_case_acronyms)]
#[allow(non_snake_case)]
pub struct MCTS {
    model: Model,
    rng: SmallRng,
    alpha: f32,
    eps: f32,
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
            rng: SmallRng::from_entropy(),
            alpha: 0.35,
            eps: 0.25,
            cpuct,
            num_simulation,
            Qsa: FxHashMap::default(),
            Nsa: FxHashMap::default(),
            Ns: FxHashMap::default(),
            Ps: FxHashMap::default(),
        }
    }
    pub fn clear_cache(&mut self) {
        self.Qsa = FxHashMap::default();
        self.Nsa = FxHashMap::default();
        self.Ns = FxHashMap::default();
        self.Ps = FxHashMap::default();
    }
    pub fn search(&mut self, board: Board) -> Result<Vec<f32>> {
        let _ = self._search(board)?;
        let state = if board.turn == Stone::Black {
            (board.black, board.white)
        } else {
            (board.white, board.black)
        };
        let eps = self.eps; // TODO selfの借用が回避できない
        let alpha = self.alpha; // TODO selfの借用が回避できない
        if let Some(p) = self.Ps.get_mut(&state) {
            *p = p.mapv(|x| (1.0 - eps) * x)
                + Array1::from_vec(
                    Dirichlet::new(
                        &board
                            .get_legal_moves()
                            .to_map()
                            .iter()
                            .flatten()
                            .map(|b| (*b as u8) as f32 * alpha + f32::EPSILON)
                            .collect::<Vec<f32>>(),
                    )?
                    .sample(&mut self.rng),
                )
                .mapv(|x| x * eps);
        }
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

    fn _search(&mut self, mut board: Board) -> Result<f32> {
        let player = board.turn;
        let state = if board.turn == Stone::Black {
            (board.black, board.white)
        } else {
            (board.white, board.black)
        };
        // let random_ordering = || {
        //     if self.rng.gen_bool(0.5) {
        //         Ordering::Greater
        //     } else {
        //         Ordering::Less
        //     }
        // };
        Ok(if let Some(p) = self.Ps.get(&state) {
            // if p.iter().any(|x| x.is_nan()) {
            //     println!("{}", &p);
            // }
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
                .max_by(|a, b| {
                    a.1.partial_cmp(&b.1).unwrap()
                    // TODO 借用の問題がある
                    // .then_with(random_ordering)
                })
                .unwrap();
            board.put(best.1)?;
            let v = if board.finished() {
                game_result(&board, player) as f32
            } else if player == board.turn {
                self._search(board)?
            } else {
                -self._search(board)?
            };
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
            v
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
            if policy.sum() <= 0.0 {
                policy += &mask;
            }
            policy /= policy.sum();
            self.Ps.insert(state, policy);
            self.Ns.insert(state, 0);
            v
        })
    }
}
