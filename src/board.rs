#![allow(dead_code)]
use std::{fmt, vec};

const SIZE: usize = 8;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stone {
    Whilte,
    Black,
}
impl Stone {
    pub fn reverse(&self) -> Stone {
        if *self == Stone::Black {
            Stone::Whilte
        } else {
            Stone::Black
        }
    }
}
const D: [(i64, i64); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

pub struct Square {
    pub x: usize,
    pub y: usize,
}
impl Square {
    fn new(x: usize, y: usize) -> Square {
        Square { x, y }
    }
    pub fn parse(input: String) -> Result<Square, String> {
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
        Ok(Square { x, y })
    }
}
fn add_coord_and_direction(x: usize, y: usize, dx: i64, dy: i64) -> Result<(usize, usize), String> {
    let a = x as i64 + dx;
    let b = y as i64 + dy;
    if !(0 <= a && a < SIZE as i64 && 0 <= b && b < SIZE as i64) {
        return Err("Out of range".to_string());
    }
    Ok((a as usize, b as usize))
}
pub struct Board([[Option<Stone>; SIZE]; SIZE]);

const BLACK_STONE_STRING: &str = "⚪️";
const WHITE_STONE_STRING: &str = "⚫️";

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = vec!["   a b c d e f g h".to_string()];
        for (i, stone_line) in self.0.iter().enumerate() {
            let line: String = stone_line
                .iter()
                .map(|x| match *x {
                    Some(Stone::Black) => BLACK_STONE_STRING,
                    Some(Stone::Whilte) => WHITE_STONE_STRING,
                    None => "　",
                })
                .collect();
            lines.push(format!(" {}{}", i, line))
        }
        write!(f, "{}", lines.join("\n"))
    }
}
impl Board {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut b = [[None; SIZE]; SIZE];
        b[3][3] = Some(Stone::Whilte);
        b[4][4] = Some(Stone::Whilte);
        b[3][4] = Some(Stone::Black);
        b[4][3] = Some(Stone::Black);
        Board(b)
    }
    pub fn is_available_square(&self, player: Stone, x: usize, y: usize) -> bool {
        if self.0[y][x].is_some() {
            return false;
        }
        for &(dx, dy) in &D {
            if let Ok((mut a, mut b)) = add_coord_and_direction(x, y, dx, dy) {
                if self.0[b][a] != Some(player.reverse()) {
                    continue;
                }
                while let Ok((next_a, next_b)) = add_coord_and_direction(a, b, dx, dy) {
                    a = next_a;
                    b = next_b;
                    if self.0[b][a].is_none() {
                        break;
                    }
                    if self.0[b][a] == Some(player) {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub fn get_available_squares(&self, player: Stone) -> Vec<(usize, usize)> {
        let mut available_squares = vec![];
        for i in 0..SIZE {
            for j in 0..SIZE {
                if self.is_available_square(player, i, j) {
                    available_squares.push((i, j));
                }
            }
        }
        available_squares
    }
    pub fn put(&mut self, stone: Stone, x: usize, y: usize) -> Result<(), String> {
        let mut reverse_stones = vec![];
        for &(dx, dy) in &D {
            let mut tmp_reverse_stones = vec![];
            if let Ok((mut a, mut b)) = add_coord_and_direction(x, y, dx, dy) {
                if self.0[b][a] != Some(stone.reverse()) {
                    continue;
                }
                tmp_reverse_stones.push((a, b));
                while let Ok((next_a, next_b)) = add_coord_and_direction(a, b, dx, dy) {
                    a = next_a;
                    b = next_b;
                    if self.0[b][a].is_none() {
                        break;
                    }
                    if self.0[b][a] == Some(stone) {
                        reverse_stones.append(&mut tmp_reverse_stones);
                        break;
                    }
                    tmp_reverse_stones.push((a, b));
                }
            }
        }
        if reverse_stones.is_empty() {
            Err(format!("Stone {:?} can't put stone. ({}, {})", stone, x, y))
        } else {
            self.0[y][x] = Some(stone);
            for (x, y) in reverse_stones {
                if let Some(stone) = &mut self.0[y][x] {
                    *stone = stone.reverse();
                } else {
                    unreachable!()
                }
            }
            Ok(())
        }
    }
}

#[derive(Default)]
pub struct StoneCounter {
    black: usize,
    white: usize
}

impl StoneCounter {
    pub fn count(board: &Board) -> StoneCounter {
        let mut counter = StoneCounter::default();
        for i in 0..SIZE {
            for j in 0..SIZE {
                match board.0[i][j] {
                    Some(Stone::Black) => counter.black += 1,
                    Some(Stone::Whilte) => counter.white += 1,
                    None => {}
                }
            }
        }
        counter
    }
}