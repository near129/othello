use std::{fmt, vec};

pub const SIZE: usize = 8;
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
impl Default for Stone {
    fn default() -> Self {
        Stone::Black
    }
}
#[derive(Default)]
pub struct StoneCount {
    black: usize,
    white: usize,
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

fn add_coord_and_direction(x: usize, y: usize, dx: i64, dy: i64) -> Result<(usize, usize), String> {
    let a = x as i64 + dx;
    let b = y as i64 + dy;
    if !(0 <= a && a < SIZE as i64 && 0 <= b && b < SIZE as i64) {
        return Err("Out of range".to_string());
    }
    Ok((a as usize, b as usize))
}
fn check_coord(x: usize, y: usize) -> Result<(), String> {
    if !(x < SIZE && y < SIZE) {
        return Err("Out of range".to_string());
    }
    Ok(())
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
    pub fn new() -> Self {
        let mut b = [[None; SIZE]; SIZE];
        b[3][3] = Some(Stone::Whilte);
        b[4][4] = Some(Stone::Whilte);
        b[3][4] = Some(Stone::Black);
        b[4][3] = Some(Stone::Black);
        Board(b)
    }
    pub fn init(&mut self) {
        let mut b = [[None; SIZE]; SIZE];
        b[3][3] = Some(Stone::Whilte);
        b[4][4] = Some(Stone::Whilte);
        b[3][4] = Some(Stone::Black);
        b[4][3] = Some(Stone::Black);
        self.0 = b;
    }
    pub fn get_board(&self) -> &[[Option<Stone>; SIZE]; SIZE] {
        &self.0
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
    pub fn get_available_squares(&self, player: Stone) -> [[bool ; SIZE]; SIZE] {
        let mut available_squares = [[false; SIZE]; SIZE];
        for i in 0..SIZE {
            for j in 0..SIZE {
                if self.is_available_square(player, i, j) {
                    available_squares[j][i] = true;
                }
            }
        }
        available_squares
    }
    pub fn put(&mut self, stone: Stone, x: usize, y: usize) -> Result<(), String> {
        check_coord(x, y)?;
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
    pub fn count_stone(&self) -> StoneCount {
        let mut counter = StoneCount::default();
        for i in 0..SIZE {
            for j in 0..SIZE {
                match self.0[i][j] {
                    Some(Stone::Black) => counter.black += 1,
                    Some(Stone::Whilte) => counter.white += 1,
                    None => {}
                }
            }
        }
        counter
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
