use serde::Deserialize;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ChessPiece {
    pub piece: Piece,
    pub color: Color,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Piece {
    KING,
    QUEEN,
    ROOK,
    BISHOP,
    KNIGHT,
    PAWN,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Color {
    BLACK,
    WHITE,
}

impl Color {
    pub fn to_str(&self) -> String {
        match self {
            Color::WHITE => "white".to_string(),
            Color::BLACK => "black".to_string(),
        }
    }
    pub fn opposite_color(&self) -> String {
        match self {
            Color::WHITE => "black".to_string(),
            Color::BLACK => "white".to_string(),
        }
    }
}
