pub mod chess_piece;
pub mod validation;

#[cfg(test)]
mod full_game_tests;

use crate::game::chess_piece::{ChessPiece, Color, Piece};
use crate::utils::convert_notation::{get_promotion_piece, get_squares_from_notation};
use crate::utils::error::{CHECK_ERROR, NO_PIECE_SELECTED_ERROR};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use self::validation::bishop::validate_bishop_move;
use self::validation::check_mate::{can_be_captured_by, can_king_be_captured_after_move, is_mate};
use self::validation::king::validate_king_move;
use self::validation::knight::validate_knight_move;
use self::validation::pawn::validate_pawn_move;
use self::validation::queen::validate_queen_move;
use self::validation::rook::validate_rook_move;

#[derive(Clone, Debug)]
pub struct Game {
    pub id: Uuid,
    pub admin_color: Color,
    pub game_result: Option<GameResult>,
    pub turn_number: u32,
    pub next_to_move: Color,
    pub previous_move: String,
    pub previous_move_was_enpassant: bool,
    pub can_castle: CastlingRights,
    pub can_en_passant: bool,
    pub king_position: KingPosition,
    pub field: Vec<Vec<Option<ChessPiece>>>,
}

#[derive(Clone, Debug)]
pub struct CastlingRights {
    pub white_can_short_castle: bool,
    pub white_can_long_castle: bool,
    pub black_can_short_castle: bool,
    pub black_can_long_castle: bool,
}

#[derive(Clone, Debug)]
pub struct KingPosition {
    pub white_king_position: (usize, usize),
    pub black_king_position: (usize, usize),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
pub enum GameResult {
    WhiteWon,
    BlackWon,
}
impl GameResult {
    pub fn to_str(&self) -> String {
        match self {
            GameResult::WhiteWon => "1-0".to_string(),
            GameResult::BlackWon => "0-1".to_string(),
        }
    }
}

impl Game {
    pub fn new(uuid: Uuid, color: Color) -> Game {
        create_new_game(uuid, color)
    }
    pub fn validate_and_make_move(
        &mut self,
        algebraic_from: &str,
        algebraic_to: &str,
        promotion_ch: char,
    ) -> Result<(), &'static str> {
        self.validate_move(algebraic_from, algebraic_to, promotion_ch)?;
        self.make_move(algebraic_from, algebraic_to, promotion_ch);

        Ok(())
    }
    pub fn validate_move(
        &self,
        algebraic_from: &str,
        algebraic_to: &str,
        promotion_ch: char,
    ) -> Result<(), &'static str> {
        let (from, to) = get_squares_from_notation(algebraic_from, algebraic_to)?;

        // check if the move is valid
        match self.field[from.0][from.1] {
            None => return Err(NO_PIECE_SELECTED_ERROR),
            Some(x) => match x.piece {
                Piece::BISHOP => validate_bishop_move(from, to, &self)?,
                Piece::ROOK => validate_rook_move(from, to, &self)?,
                Piece::QUEEN => validate_queen_move(from, to, &self)?,
                Piece::KNIGHT => validate_knight_move(from, to, &self)?,
                Piece::PAWN => validate_pawn_move(from, to, promotion_ch, &self)?,
                Piece::KING => validate_king_move(from, to, &self)?,
            },
        };

        // check if the move would put your king in check
        if can_king_be_captured_after_move(&self, algebraic_from, algebraic_to, promotion_ch).len()
            != 0
        {
            return Err(CHECK_ERROR);
        }

        Ok(())
    }
    pub fn make_move(&mut self, algebraic_from: &str, algebraic_to: &str, promotion_ch: char) {
        // we can unwrap here since we perform this function in the validation function as well
        let (from, to) = get_squares_from_notation(algebraic_from, algebraic_to).unwrap();
        self.can_en_passant = false;
        self.previous_move_was_enpassant = false;

        // Add x in case we capture
        let move_with_capture = self.field[to.0][to.1].is_some();
        if move_with_capture {
            self.previous_move = "x".to_string();
        } else {
            self.previous_move = "".to_string();
        }

        // ugly, but we need to check for en passant before making the actual move
        if self.field[to.0][to.1].is_none()
            && self.field[from.0][from.1].unwrap().piece == Piece::PAWN
            && from.1 != to.1
        {
            self.field[from.0][to.1] = None;
            self.previous_move = "x".to_string();
            self.previous_move_was_enpassant = true;
        }

        // Target square
        self.previous_move.push_str(algebraic_to);

        // move to new square
        self.field[to.0][to.1] = self.field[from.0][from.1];
        self.field[from.0][from.1] = None;

        // for some pieces we need custom logic
        match self.field[to.0][to.1].unwrap().piece {
            Piece::KING => self.make_king_move(from, to),
            Piece::PAWN => self.make_pawn_move(from, to, promotion_ch),
            Piece::QUEEN => self.previous_move.insert(0, 'Q'),
            Piece::ROOK => self.make_rook_move(from),
            Piece::BISHOP => self.previous_move.insert(0, 'B'),
            Piece::KNIGHT => self.previous_move.insert(0, 'N'),
        }

        // change turn and add check to notation if necessary
        match self.next_to_move {
            Color::BLACK => {
                self.next_to_move = Color::WHITE;
                if can_be_captured_by(Color::BLACK, self.king_position.white_king_position, &self)
                    .len()
                    > 0
                {
                    self.previous_move.push('+');
                }
            }
            Color::WHITE => {
                self.next_to_move = Color::BLACK;
                if can_be_captured_by(Color::WHITE, self.king_position.black_king_position, &self)
                    .len()
                    > 0
                {
                    self.previous_move.push('+');
                }
                self.turn_number += 1;
            }
        }

        if is_mate(self) {
            match self.next_to_move {
                Color::WHITE => {
                    self.game_result = Some(GameResult::BlackWon);
                }
                Color::BLACK => {
                    self.game_result = Some(GameResult::WhiteWon);
                }
            }
        }
    }
    fn make_rook_move(&mut self, from: (usize, usize)) {
        self.previous_move.insert(0, 'R');

        // Take away castling rights if necessary
        if self.next_to_move == Color::BLACK {
            if from.0 == 0 && from.1 == 0 {
                self.can_castle.black_can_long_castle = false;
            }
            if from.0 == 0 && from.1 == 7 {
                self.can_castle.black_can_short_castle = false;
            }
        }
        if self.next_to_move == Color::WHITE {
            if from.0 == 7 && from.1 == 0 {
                self.can_castle.white_can_long_castle = false;
            }
            if from.0 == 7 && from.1 == 7 {
                self.can_castle.white_can_short_castle = false;
            }
        }
    }
    fn make_king_move(&mut self, from: (usize, usize), to: (usize, usize)) {
        self.previous_move.insert(0, 'K');
        // Check if castling move
        match (from, to) {
            ((0, 4), (0, 6)) => {
                self.field[0][5] = self.field[0][7];
                self.field[0][7] = None;
                self.previous_move = "0-0".to_string();
            }
            ((0, 4), (0, 2)) => {
                self.field[0][3] = self.field[0][0];
                self.field[0][0] = None;
                self.previous_move = "0-0-0".to_string();
            }
            ((7, 4), (7, 6)) => {
                self.field[7][5] = self.field[7][7];
                self.field[7][7] = None;
                self.previous_move = "0-0".to_string();
            }
            ((7, 4), (7, 2)) => {
                self.field[7][3] = self.field[7][0];
                self.field[7][0] = None;
                self.previous_move = "0-0-0".to_string();
            }
            _ => (),
        }

        // Change king position and castling rights
        match self.next_to_move {
            Color::BLACK => {
                self.king_position.black_king_position = to;
                self.can_castle.black_can_long_castle = false;
                self.can_castle.black_can_short_castle = false;
            }
            Color::WHITE => {
                self.king_position.white_king_position = to;
                self.can_castle.white_can_long_castle = false;
                self.can_castle.white_can_short_castle = false;
            }
        }
    }
    fn make_pawn_move(&mut self, from: (usize, usize), to: (usize, usize), promotion_ch: char) {
        // Check if promotion move
        if to.0 == 7 || to.0 == 0 {
            // unwrap due ot already being checked in validation function
            let promotion_piece = get_promotion_piece(promotion_ch).unwrap();
            self.field[to.0][to.1] = Some(ChessPiece {
                piece: promotion_piece,
                color: self.next_to_move,
            });
            self.previous_move.push('=');
            self.previous_move.push(promotion_ch);
        }

        // Set en passant rights if pawn moved 2 squares
        if (from.0 as i32 - to.0 as i32).abs() == 2 {
            self.can_en_passant = true;
        }
    }
}

fn create_new_game(uuid: Uuid, color: Color) -> Game {
    Game {
        id: uuid,
        admin_color: color,
        game_result: None,
        turn_number: 0,
        previous_move: "".to_string(),
        previous_move_was_enpassant: false,
        next_to_move: Color::WHITE,
        can_castle: CastlingRights {
            white_can_short_castle: true,
            white_can_long_castle: true,
            black_can_short_castle: true,
            black_can_long_castle: true,
        },
        can_en_passant: false,
        king_position: {
            KingPosition {
                white_king_position: (7, 4),
                black_king_position: (0, 4),
            }
        },
        field: vec![
            vec![
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::ROOK,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::KNIGHT,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::BISHOP,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::QUEEN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::KING,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::BISHOP,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::KNIGHT,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::ROOK,
                }),
            ],
            vec![
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::BLACK,
                    piece: Piece::PAWN,
                }),
            ],
            vec![None; 8],
            vec![None; 8],
            vec![None; 8],
            vec![None; 8],
            vec![
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::PAWN,
                }),
            ],
            vec![
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::ROOK,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::KNIGHT,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::BISHOP,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::QUEEN,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::KING,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::BISHOP,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::KNIGHT,
                }),
                Some(ChessPiece {
                    color: Color::WHITE,
                    piece: Piece::ROOK,
                }),
            ],
        ],
    }
}
