use crate::{
    game::{chess_piece::Color, Game},
    utils::{
        convert_notation::{get_notation_from_square, get_promotion_piece},
        error::{CAPTURE_OWN_PIECE_ERROR, GENERAL_ERROR, PIECE_IN_THE_WAY_ERROR, PROMOTION_ERROR},
    },
};

pub fn validate_pawn_move(
    from: (usize, usize),
    to: (usize, usize),
    promotion_ch: char,
    game: &Game,
) -> Result<(), &'static str> {
    let row_diff = from.0 as i32 - to.0 as i32;
    let col_diff = from.1 as i32 - to.1 as i32;

    match (row_diff, col_diff, game.next_to_move) {
        // standard pawn move
        (1, 0, Color::WHITE) | (-1, 0, Color::BLACK) => {
            if game.field[to.0][to.1].is_some() {
                return Err(PIECE_IN_THE_WAY_ERROR);
            }
        }
        // 2 squares pawn move
        (-2, 0, Color::BLACK) => {
            if from.0 != 1 || to.0 != 3 {
                return Err(GENERAL_ERROR);
            }
            if game.field[to.0][to.1].is_some() || game.field[from.0 + 1][to.1].is_some() {
                return Err(PIECE_IN_THE_WAY_ERROR);
            }
        }
        (2, 0, Color::WHITE) => {
            if from.0 != 6 || to.0 != 4 {
                return Err(GENERAL_ERROR);
            }
            if game.field[to.0][to.1].is_some() || game.field[from.0 - 1][to.1].is_some() {
                return Err(PIECE_IN_THE_WAY_ERROR);
            }
        }
        // captures
        (1, -1, Color::WHITE)
        | (1, 1, Color::WHITE)
        | (-1, -1, Color::BLACK)
        | (-1, 1, Color::BLACK) => {
            // check for wrong capture
            if let Some(piece) = game.field[to.0][to.1] {
                if piece.color == game.next_to_move {
                    return Err(CAPTURE_OWN_PIECE_ERROR);
                }
            }

            // check for en passant
            if game.field[to.0][to.1].is_none()
                && (!game.can_en_passant
                    || game.previous_move != get_notation_from_square((from.0, to.1))?)
            {
                return Err(GENERAL_ERROR);
            }
        }
        _ => return Err(GENERAL_ERROR),
    }

    // check for promotion moves
    let promotion_piece = get_promotion_piece(promotion_ch);
    if (to.0 == 0 || to.0 == 7) && promotion_piece.is_none() {
        return Err(PROMOTION_ERROR);
    }

    Ok(())
}

#[cfg(test)]
mod test_pawn {
    use uuid::Uuid;

    use crate::{
        game::chess_piece::{Color, Piece},
        game::ChessPiece,
        game::Game,
    };

    #[test]
    fn test_pawn_move() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let _ = game
            .validate_and_make_move("b2", "b4", ' ')
            .expect("Expected pawn move to be performed");

        assert_eq!(game.field[6][1], None);
        assert_eq!(
            game.field[4][1],
            Some({
                ChessPiece {
                    piece: Piece::PAWN,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "b4");
    }

    #[test]
    fn test_pawn_move_with_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let _ = game
            .validate_and_make_move("d2", "d4", ' ')
            .expect("Expected pawn move to be performed");
        let _ = game
            .validate_and_make_move("d7", "d5", ' ')
            .expect("Expected pawn move to be performed");
        let _ = game
            .validate_and_make_move("c2", "c4", ' ')
            .expect("Expected pawn move to be performed");
        let _ = game
            .validate_and_make_move("e7", "e6", ' ')
            .expect("Expected pawn move to be performed");
        let _ = game
            .validate_and_make_move("c4", "d5", ' ')
            .expect("Expected pawn move to be performed");

        assert_eq!(game.field[6][2], None);
        assert_eq!(game.field[6][3], None);
        assert_eq!(game.field[1][3], None);
        assert_eq!(game.field[1][4], None);
        assert_eq!(
            game.field[3][3],
            Some({
                ChessPiece {
                    piece: Piece::PAWN,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "xd5");
    }

    #[test]
    fn test_pawn_move_with_en_passant() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let _ = game
            .validate_and_make_move("d2", "d4", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, true);
        let _ = game
            .validate_and_make_move("h7", "h6", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, false);
        let _ = game
            .validate_and_make_move("d4", "d5", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, false);
        let _ = game
            .validate_and_make_move("e7", "e5", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, true);
        let _ = game
            .validate_and_make_move("d5", "e6", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, false);

        assert_eq!(game.field[6][3], None);
        assert_eq!(game.field[1][4], None);
        assert_eq!(game.field[3][4], None);
        assert_eq!(
            game.field[2][4],
            Some({
                ChessPiece {
                    piece: Piece::PAWN,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "xe6");
    }

    #[test]
    fn test_pawn_move_with_incorrect_en_passant() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let _ = game
            .validate_and_make_move("d2", "d4", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, true);
        let _ = game
            .validate_and_make_move("h7", "h6", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, false);
        let _ = game
            .validate_and_make_move("d4", "d5", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, false);
        let _ = game
            .validate_and_make_move("g7", "g5", ' ')
            .expect("Expected pawn move to be performed");
        assert_eq!(game.can_en_passant, true);
        let val = game.validate_and_make_move("d5", "e6", ' ');
        if val.is_ok() {
            panic!("Expected pawn move to fail due to having en passant rights but no pawn present on capture square");
        }
    }

    #[test]
    fn test_pawn_move_with_wrong_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[5][2] = Some(ChessPiece {
            piece: Piece::PAWN,
            color: Color::WHITE,
        });
        let val = game.validate_and_make_move("b2", "c3", ' ');
        if val.is_ok() {
            panic!("Expected pawn move to fail due to your own piece being captured");
        }
    }

    #[test]
    fn test_pawn_move_with_piece_in_the_way() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[5][7] = Some(ChessPiece {
            piece: Piece::QUEEN,
            color: Color::BLACK,
        });
        let val = game.validate_and_make_move("h2", "h4", ' ');
        if val.is_ok() {
            panic!("Expected pawn move to fail due to a piece being in the way");
        }
    }

    #[test]
    fn test_pawn_invalid_two_square_move() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let _ = game
            .validate_and_make_move("h2", "h3", ' ')
            .expect("Expected pawn move to be performed");
        let _ = game
            .validate_and_make_move("d7", "d5", ' ')
            .expect("Expected pawn move to be performed");
        let val = game.validate_and_make_move("h3", "h5", ' ');
        if val.is_ok() {
            panic!("Expected pawn move to fail due to the pawn already being moved from the starting square");
        }
    }
}
