use std::{env, fs::create_dir, iter};

use anyhow::Result;
use indicatif::ProgressIterator;
use ndarray::{Array1, Array3, Axis};
use ndarray_npy::write_npy;
use othello::{players::AlphaZeroPlayer, Board, Position, Stone, StoneCount, SIZE, UPPER_LEFT};

fn create_board_array(board: &Board) -> Array3<u8> {
    let mut board_array = Array3::zeros((2, SIZE, SIZE));
    let (black_idx, white_idx) = if board.turn == Stone::Black {
        (0, 1)
    } else {
        (1, 0)
    };
    for i in 0..SIZE * SIZE {
        let pos = UPPER_LEFT >> i;
        if board.black & pos != 0 {
            board_array[[black_idx, i / SIZE, i % SIZE]] = 1;
        } else if board.white & pos != 0 {
            board_array[[white_idx, i / SIZE, i % SIZE]] = 1;
        }
    }
    board_array
}

const N: usize = 500;
const NUM_SIMULATION: usize = 5000;
fn main() -> Result<()> {
    let cwd = env::current_dir()?;
    let p = env::args().nth(1).expect("must set output path");
    let output_path = cwd.join(p);
    let mut player = AlphaZeroPlayer::new(
        "/Users/near129/dev/python/othello-alphazero/models/model.onnx",
        NUM_SIMULATION,
    );
    let mut states = vec![];
    let mut policy = vec![];
    let mut values = vec![];
    let mut _win_cnt = 0;
    println!("Simulation start");
    for i in (0..N).progress() {
        // println!("{}", i);
        print!("\r{:3}/{}", i, N);
        let mut board = Board::new();
        let mut turn = 0;
        while !board.finished() {
            // println!("{:?}", board.turn);
            // println!("{}", board);
            states.push(create_board_array(&board));
            let ret = player.mcts.search(board)?;
            let idx = ret
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0;
            policy.push(Array1::from_shape_vec(SIZE * SIZE, ret)?);
            let pos = Position(UPPER_LEFT >> idx);
            board.put(pos)?;
            turn += 1;
        }
        let StoneCount { black, white } = board.count_stone();
        let _result = if black > white {
            values.extend(iter::repeat(1).take(turn));
            _win_cnt += 1;
            "Win"
        } else {
            values.extend(iter::repeat(-1).take(turn));
            "Lose"
        };
        player.mcts.clear_cache();
    }
    let states = ndarray::stack(
        Axis(0),
        &states.iter().map(|s| s.view()).collect::<Vec<_>>(),
    )?;
    let policy = ndarray::stack(
        Axis(0),
        &policy.iter().map(|s| s.view()).collect::<Vec<_>>(),
    )?;
    // let policy = Array2::from_shape_vec((policy.len(), SIZE * SIZE), policy)?;
    let values = Array1::from_shape_vec(values.len(), values)?;
    // println!("{}", win_cnt as f32 / N as f32);
    write_npy(output_path.join("states.npy"), &states)?;
    write_npy(output_path.join("policy.npy"), &policy)?;
    write_npy(output_path.join("values.npy"), &values)?;
    println!("{}", values.len());
    Ok(())
}
