use std::ops::{Shl, Shr};


pub fn legal_move(player: u64, opponent: u64) -> u64 {
    let masks = [
        (1, 0x7e7e7e7e7e7e7e7e), // 左右
        (7, 0x007e7e7e7e7e7e00), // 右上、左下
        (8, 0x00ffffffffffff00), // 上下
        (9, 0x007e7e7e7e7e7e00), // 左上、右下
    ];
    let shifts = [Shr::shr, Shl::shl];
    let mut candidate = 0;

    for (n_shifts, mask) in masks.iter() {
        let mask = mask & opponent;

        for shift in shifts.iter() {
            let mut bits = mask & shift(player, n_shifts);
            for _ in 0..5 {
                bits |= mask & shift(bits, n_shifts);
            }
            candidate |= shift(bits, n_shifts);
        }
    }

    candidate & !(player | opponent)
}
pub fn reverse(player: u64, opponent: u64, position: u64) -> u64 {
    let masks: [(i32, u64); 4] = [
        (1, 0xfefefefefefefefe), (7, 0x7f7f7f7f7f7f7f00),
        (8, 0xffffffffffffff00), (9, 0xfefefefefefefe00),
    ];
    let shifts = [Shl::shl, Shr::shr];
    let mut rev = 0;

    for (n_shifts, mut mask) in masks.iter() {
        for shift in shifts.iter() {
            let mut r = 0;
            let mut pos = mask & shift(position, n_shifts);
            while pos & opponent != 0 {
                r |= pos;
                pos = mask & shift(pos, n_shifts);
            }
            if pos & player != 0 {
                rev |= r;
            }

            mask >>= n_shifts;
        }
    }

    rev
}

pub fn put (player: u64, opponent: u64, position: u64) -> (u64, u64) {
    let rev = reverse(player, opponent, position);
    let player = player ^ (position | rev);
    let opponent = opponent ^ rev;

    (player, opponent)
}