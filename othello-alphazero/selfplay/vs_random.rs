use std::env;

use anyhow::Result;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar};
#[allow(unused_imports)]
use othello::{Board, Stone, StoneCount, players::{AlphaZeroPlayer, GreedyPlayer, MCTSPlayer, Player, RandomPlayer}};
use tokio::spawn;
use tokio::task::spawn_blocking;

#[allow(dead_code)]
const NUM_SIMULATION: usize = 125;
async fn battle(idx: usize, pb: ProgressBar) -> Result<usize> {
    // let mut player1 = AlphaZeroPlayer::new_from_model_path("/Users/near129/dev/rust/othello/othello-alphazero/train_model/othellonet.onnx", NUM_SIMULATION);
    let mut player1 = AlphaZeroPlayer::new_from_model_path("models/model.onnx", NUM_SIMULATION);
    // let mut player1 = MCTSPlayer::new(1.4, 5000);
    let mut player2 = GreedyPlayer::default();
    // let mut player2 = RandomPlayer::default();
    let player1_stone = if idx % 2 == 0 {
        Stone::Black
    } else {
        Stone::White
    };
    let mut board = Board::new();
    // let _ = player2.init_search(50000, board);
    while !board.finished() {
        // println!("{:?}", board.turn);
        // println!("{}", board);
        let pos = if board.turn == player1_stone {
            player1.find_move(&board)
        } else {
            player2.find_move(&board)
        }?;
        board.put(pos)?;
    }
    let StoneCount { black, white } = board.count_stone();
    pb.inc(1);
    if (player1_stone == Stone::Black && black >= white)
        || (player1_stone == Stone::White && white >= black)
    {
        Ok(1)
    } else {
        Ok(0)
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let num_simulation: usize = args[1].parse().unwrap();

    let m = MultiProgress::new();
    let pb = m.add(ProgressBar::new(num_simulation as u64));
    let mut worker = vec![];
    pb.println("alpahzero vs random battle");
    for i in 0..num_simulation {
        worker.push(spawn(battle(i, pb.clone())));
    }
    let mp = spawn_blocking(move || m.join().unwrap());
    let result: usize = join_all(worker)
        .await
        .into_iter()
        .collect::<std::result::Result<Result<Vec<_>>, _>>()??
        .iter()
        .sum();
    pb.finish_and_clear();
    let _ = mp.await;
    println!("{}", result as f32 / num_simulation as f32);
    Ok(())
}
