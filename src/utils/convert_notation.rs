use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;

use crate::game::chess_piece::Piece;

use super::error::{INVALID_FROM_FIELD, INVALID_TO_FIELD, SQUARE_OUT_OF_BOUNDS_ERROR};

pub fn get_squares_from_notation(
    from: &str,
    to: &str,
) -> Result<((usize, usize), (usize, usize)), &'static str> {
    let square_mapping = NOTATION_TO_SQUARE_MAP.lock().unwrap();
    let from_col = square_mapping
        .get(&from.chars().nth(0).ok_or(INVALID_FROM_FIELD)?)
        .ok_or(SQUARE_OUT_OF_BOUNDS_ERROR)?;
    let to_col = square_mapping
        .get(&to.chars().nth(0).ok_or(INVALID_TO_FIELD)?)
        .ok_or(SQUARE_OUT_OF_BOUNDS_ERROR)?;

    let from_row = 8 - from
        .chars()
        .nth(1)
        .ok_or(INVALID_FROM_FIELD)?
        .to_digit(10)
        .ok_or(SQUARE_OUT_OF_BOUNDS_ERROR)?;
    let to_row = 8 - to
        .chars()
        .nth(1)
        .ok_or(INVALID_TO_FIELD)?
        .to_digit(10)
        .ok_or(SQUARE_OUT_OF_BOUNDS_ERROR)?;

    Ok(((from_row as usize, *from_col), (to_row as usize, *to_col)))
}

pub fn get_notation_from_square(square: (usize, usize)) -> Result<String, &'static str> {
    let notation_mapping = SQUARE_TO_NOTATION_MAP.lock().unwrap();
    let mut notation: String = "".to_string();

    // column
    notation.push(
        *notation_mapping
            .get(&square.1)
            .ok_or(SQUARE_OUT_OF_BOUNDS_ERROR)?,
    );
    // row
    notation.push_str(&(8 - square.0).to_string());

    Ok(notation)
}

pub fn get_promotion_piece(promotion_ch: char) -> Option<Piece> {
    let promotion_mapping = PROMOTION_MAP.lock().unwrap();
    promotion_mapping.get(&promotion_ch).copied()
}

static NOTATION_TO_SQUARE_MAP: Lazy<Mutex<HashMap<char, usize>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('a', 0);
    map.insert('b', 1);
    map.insert('c', 2);
    map.insert('d', 3);
    map.insert('e', 4);
    map.insert('f', 5);
    map.insert('g', 6);
    map.insert('h', 7);
    Mutex::new(map)
});

static SQUARE_TO_NOTATION_MAP: Lazy<Mutex<HashMap<usize, char>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(0, 'a');
    map.insert(1, 'b');
    map.insert(2, 'c');
    map.insert(3, 'd');
    map.insert(4, 'e');
    map.insert(5, 'f');
    map.insert(6, 'g');
    map.insert(7, 'h');
    Mutex::new(map)
});

static PROMOTION_MAP: Lazy<Mutex<HashMap<char, Piece>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('Q', Piece::QUEEN);
    map.insert('R', Piece::ROOK);
    map.insert('B', Piece::BISHOP);
    map.insert('N', Piece::KNIGHT);
    Mutex::new(map)
});
