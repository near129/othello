use anyhow::Result;
use othello::{
    players::{AlphaZeroPlayer, Player, RandomPlayer},
    Board, Stone, StoneCount,
};

const N: usize = 10;
const NUM_SIMULATION: usize = 10000;
fn main() -> Result<()> {
    let mut player1 = AlphaZeroPlayer::new(
        "/Users/near129/dev/python/othello-alphazero/models/model.onnx",
        NUM_SIMULATION,
    );
    let mut player2 = RandomPlayer::new();
    let mut win_cnt = 0;
    println!("Simulation start");
    for i in 0..N {
        let player1_stone = if i % 2 == 0 {
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
        //     "Lose"
        // };
        let result = if (player1_stone == Stone::Black && black >= white)
            || (player1_stone == Stone::Whilte && white >= black)
        {
            win_cnt += 1;
            "Win"
        } else {
            "Lose"
        };
        println!("{:3}: {:4}  {:2} - {:2}", i, result, black, white);
        player1.mcts.clear_cache();
    }
    println!("win lating: {}", win_cnt as f32 / N as f32);
    Ok(())
}
