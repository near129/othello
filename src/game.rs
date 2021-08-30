// use crate::board::{Board, SIZE, Stone, StoneCount};

// #[derive(Debug)]
// pub struct TurnEndResult {
//     put: Result<(), String>,
//     finish: bool,
//     pass: bool,
// }

// #[derive(Default)]
// pub struct Game {
//     pub board: Board,
//     pub turn: Stone,
//     pub turn_count: usize,
//     pub finished: bool,
// }

// impl Game {
//     pub fn count_stone(&self) -> StoneCount {
//         self.board.count_stone()
//     }
//     pub fn get_available_squares(&self) -> Result<[[bool ; SIZE]; SIZE], String> {
//         if self.finished {
//             return Err("The game is alredy over.".to_string());
//         }
//         Ok(self.board.get_available_squares(self.turn))
//     }
//     pub fn put(&mut self, x: usize, y: usize) -> Result<(), String> {
//         if self.finished {
//             return Err("The game is alredy over.".to_string());
//         }
//         let _ = self.board.put(self.turn, x, y)?;
//         self.finished = self.finish();
//         if !self.pass() {
//             self.turn = self.turn.reverse();
//         }
//         Ok(())
//     }
//     fn finish(&self) -> bool {
//         self.board.get_available_squares(Stone::Black).is_empty()
//             && self.board.get_available_squares(Stone::Whilte).is_empty()
//     }
//     fn pass(&self) -> bool {
//         self.board
//             .get_available_squares(self.turn.reverse())
//             .is_empty()
//     }
// }
