use crate::{board::SIZE, Board, Stone, StoneCount, UPPER_LEFT};
use tract_onnx::{prelude::*, tract_hir::tract_ndarray::Array3};
pub fn input_parse(input: &str) -> Result<(usize, usize), String> {
    let input: Vec<_> = input.chars().collect();
    let is_valid = input.len() == 2 && {
        let x = input[0];
        let y = input[1];
        (x.is_alphabetic() && x as u8 - b'a' < SIZE as u8)
            && (y.is_numeric() && y as u8 - b'0' < SIZE as u8)
    };
    if !is_valid {
        return Err("Invalid input. Input must be [a-h][0-7]".to_string());
    }
    let x = (input[0] as u8 - b'a') as usize;
    let y = (input[1] as u8 - b'0') as usize;
    Ok((x, y))
}

pub fn game_result(board: &Board, player: Stone) -> i8 {
    let StoneCount { black, white } = board.count_stone();
    let res = match black.cmp(&white) {
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Less => -1,
    };
    if player == Stone::Black {
        res
    } else {
        -res
    }
}
pub fn create_board_tensor(board: &Board) -> Tensor {
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
