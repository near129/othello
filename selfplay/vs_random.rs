use std::env;

use anyhow::Result;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar};
use othello::{
    players::{AlphaZeroPlayer, Player, RandomPlayer},
    Board, Stone, StoneCount,
};
use tokio::spawn;
use tokio::task::spawn_blocking;

const NUM_SIMULATION: usize = 10000;
async fn battle(idx: usize, pb: ProgressBar) -> Result<usize> {
    let mut player1 = AlphaZeroPlayer::new_from_model_path("./models/model.onnx", NUM_SIMULATION);
    let mut player2 = RandomPlayer::new();
    let player1_stone = if idx % 2 == 0 {
        Stone::Black
    } else {
        Stone::Whilte
    };
    let mut board = Board::new();
    while !board.finished() {
        // println!("{:?}", board.turn);
        // println!("{}", board);
        let pos = if board.turn == player1_stone {
            // let pos = if board.turn == Stone::Black {
            player1.find_move(&board)
        } else {
            player2.find_move(&board)
        }?;
        board.put(pos)?;
    }
    let StoneCount { black, white } = board.count_stone();
    // let result = if black >= white {
    //     win_cnt += 1;
    //     "Win"
    // } else {
    //   "Lose"
    // };
    pb.inc(1);
    if (player1_stone == Stone::Black && black >= white)
        || (player1_stone == Stone::Whilte && white >= black)
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
    println!("win lating: {}", result as f32 / num_simulation as f32);
    Ok(())
}
