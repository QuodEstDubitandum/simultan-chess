use crate::{
    game::Game,
    utils::error::{CAPTURE_OWN_PIECE_ERROR, GENERAL_ERROR, PIECE_IN_THE_WAY_ERROR},
};

pub fn validate_rook_move(
    from: (usize, usize),
    to: (usize, usize),
    game: &Game,
) -> Result<(), &'static str> {
    let row_diff = from.0 as i32 - to.0 as i32;
    let col_diff = from.1 as i32 - to.1 as i32;

    // not even move
    match (row_diff, col_diff) {
        (0, 0) => return Err(GENERAL_ERROR),
        (0, _) => {
            let col_direction_sign = col_diff / -col_diff.abs();
            for i in 1..col_diff.abs() {
                if game.field[from.0][(from.1 as i32 + i * col_direction_sign) as usize].is_some() {
                    return Err(PIECE_IN_THE_WAY_ERROR);
                };
            }
        }
        (_, 0) => {
            let row_direction_sign = row_diff / -row_diff.abs();
            for i in 1..row_diff.abs() {
                if game.field[(from.0 as i32 + i * row_direction_sign) as usize][from.1].is_some() {
                    return Err(PIECE_IN_THE_WAY_ERROR);
                };
            }
        }
        _ => return Err(GENERAL_ERROR),
    }

    // if you capture a piece, is it of the opposite color?
    if let Some(piece) = game.field[to.0][to.1] {
        if piece.color == game.next_to_move {
            return Err(CAPTURE_OWN_PIECE_ERROR);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test_rook {
    use uuid::Uuid;

    use crate::{
        game::chess_piece::{Color, Piece},
        game::ChessPiece,
        game::Game,
    };

    #[test]
    fn test_rook_move() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[6][0] = None;
        let val = game.validate_and_make_move("a1", "a5", ' ');
        if let Err(e) = val {
            panic!("Expected rook move to be performed, got {:?}", e);
        }

        assert_eq!(game.field[7][0], None);
        assert_eq!(
            game.field[3][0],
            Some({
                ChessPiece {
                    piece: Piece::ROOK,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "Ra5");
        assert_eq!(game.can_castle.white_can_long_castle, false);
    }

    #[test]
    fn test_rook_move_with_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[1][0] = None;
        game.field[5][0] = Some(ChessPiece {
            piece: Piece::ROOK,
            color: Color::WHITE,
        });
        game.next_to_move = Color::BLACK;
        let val = game.validate_and_make_move("a8", "a3", ' ');
        if let Err(e) = val {
            panic!("Expected rook move to be performed, got {:?}", e);
        }

        assert_eq!(game.field[0][0], None);
        assert_eq!(
            game.field[5][0],
            Some({
                ChessPiece {
                    piece: Piece::ROOK,
                    color: Color::BLACK,
                }
            })
        );
        assert_eq!(game.previous_move, "Rxa3");
        assert_eq!(game.can_castle.black_can_long_castle, false);
        assert_eq!(game.can_castle.white_can_long_castle, true);
    }

    #[test]
    fn test_rook_move_with_wrong_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[7][0] = None;
        game.field[3][0] = Some(ChessPiece {
            piece: Piece::PAWN,
            color: Color::WHITE,
        });
        let val = game.validate_and_make_move("a1", "a5", ' ');
        if val.is_ok() {
            panic!("Expected rook move to fail due to your own piece being captured");
        }
    }

    #[test]
    fn test_rook_move_with_piece_in_the_way() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let val = game.validate_and_make_move("a1", "a5", ' ');
        if val.is_ok() {
            panic!("Expected rook move to fail due to a piece being in the way");
        }
    }
}
