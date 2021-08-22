use crate::board::SIZE;
pub fn input_parse(input: String) -> Result<(usize, usize), String> {
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
