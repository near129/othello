use std::cmp::Ordering;
use std::env;

use othello::Stone;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

use anyhow::Result;
use futures::future::join_all;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use ndarray::{Array1, Array3, Axis};
use ndarray_npy::write_npy;
use othello::{players::AlphaZeroPlayer, Board, Position, StoneCount, SIZE, UPPER_LEFT};
use tokio::spawn;
use tokio::task::spawn_blocking;

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

async fn simulate(
    n: usize,
    pb: ProgressBar,
    mcts_simulation: usize
) -> Result<(Vec<Array3<u8>>, Vec<Array1<f32>>, Vec<i32>)> {
    let mut rng = SmallRng::from_entropy();
    let mut player = AlphaZeroPlayer::new_from_model_path("./models/model.onnx", mcts_simulation);
    // let mut player = AlphaZeroPlayer::new_from_model_path("/Users/near129/GoggleDrive/tmp/regnet.onnx", mcts_simulation);
    let mut states = vec![];
    let mut policy = vec![];
    let mut values = vec![];
    for _ in 0..n {
        let mut board = Board::new();
        let mut tmp_values = vec![];
        let mut i = 0;
        while !board.finished() {
            tmp_values.push(if board.turn == Stone::Black { 1 } else { -1 });
            states.push(create_board_array(&board));
            let ret = player.mcts.search(board)?;
            let idx = if i <= 30 {
                *(0..SIZE * SIZE)
                    .collect::<Vec<_>>()
                    .choose_weighted(&mut rng, |&idx| ret[idx])?
            } else {
                ret.iter()
                    .enumerate()
                    .max_by(|a, b| {
                        a.1.partial_cmp(b.1).unwrap().then_with(|| {
                            if rng.gen_bool(0.5) {
                                Ordering::Greater
                            } else {
                                Ordering::Less
                            }
                        })
                    })
                    .unwrap()
                    .0
            };
            policy.push(Array1::from_shape_vec(SIZE * SIZE, ret)?);
            let pos = Position(UPPER_LEFT >> idx);
            board.put(pos)?;
            i += 1;
        }
        let StoneCount { black, white } = board.count_stone();
        let res = match black.cmp(&white) {
            Ordering::Equal => 0,
            Ordering::Greater => 1,
            Ordering::Less => -1,
        };
        values.extend(tmp_values.iter().map(|x| x * res));
        player.mcts.clear_cache();
        pb.inc(1);
    }
    Ok((states, policy, values))
}
#[tokio::main]
async fn main() -> Result<()> {
    let cwd = env::current_dir()?;
    let args: Vec<String> = env::args().collect();
    let output_path = cwd.join(&args[1]);
    let num_worker: usize = args[2].parse()?;
    let num_simulation: usize = args[3].parse()?;
    let mcts_simulation: usize = args[4].parse()?;
    let m = MultiProgress::new();
    let pb = m.add(ProgressBar::new(
        (num_simulation / num_worker * num_worker) as u64,
    ));
    pb.println("simulation start");
    let mut worker = vec![];
    for _ in 0..num_worker {
        worker.push(spawn(simulate(num_simulation / num_worker, pb.clone(), mcts_simulation)));
    }
    let mp = spawn_blocking(move || m.join_and_clear().unwrap());
    let mut result = join_all(worker)
        .await
        .into_iter()
        .collect::<std::result::Result<Result<Vec<_>>, _>>()??;
    pb.finish();
    let _ = mp.await;
    let n: usize = result.iter().map(|x| x.2.len()).sum();
    let mut states = Vec::with_capacity(n);
    let mut policy = Vec::with_capacity(n);
    let mut values = Vec::with_capacity(n);
    for (s, p, v) in result.iter_mut() {
        states.append(s);
        policy.append(p);
        values.append(v);
    }
    let states = ndarray::stack(
        Axis(0),
        &states.iter().map(|s| s.view()).collect::<Vec<_>>(),
    )?;
    let policy = ndarray::stack(
        Axis(0),
        &policy.iter().map(|s| s.view()).collect::<Vec<_>>(),
    )?;
    let values = Array1::from_shape_vec(values.len(), values)?;
    write_npy(output_path.join("states.npy"), &states)?;
    write_npy(output_path.join("policy.npy"), &policy)?;
    write_npy(output_path.join("values.npy"), &values)?;
    println!("Finished! number of data: {}", values.len());
    Ok(())
}
