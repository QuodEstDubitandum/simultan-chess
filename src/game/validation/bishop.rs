use crate::{
    game::Game,
    utils::error::{CAPTURE_OWN_PIECE_ERROR, GENERAL_ERROR, PIECE_IN_THE_WAY_ERROR},
};

pub fn validate_bishop_move(
    from: (usize, usize),
    to: (usize, usize),
    game: &Game,
) -> Result<(), &'static str> {
    let row_diff = from.0 as i32 - to.0 as i32;
    let col_diff = from.1 as i32 - to.1 as i32;

    // not even diagonal move
    if (row_diff).abs() != (col_diff).abs() || row_diff == 0 {
        return Err(GENERAL_ERROR);
    }

    // is there a piece in the way?
    let row_direction_sign = row_diff / -row_diff.abs();
    let col_direction_sign = col_diff / -col_diff.abs();
    for i in 1..row_diff.abs() {
        if game.field[(from.0 as i32 + i * row_direction_sign) as usize]
            [(from.1 as i32 + i * col_direction_sign) as usize]
            .is_some()
        {
            return Err(PIECE_IN_THE_WAY_ERROR);
        };
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
mod test_bishop {
    use uuid::Uuid;

    use crate::{
        game::chess_piece::{Color, Piece},
        game::ChessPiece,
        game::Game,
    };

    #[test]
    fn test_bishop_move() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[6][4] = None;
        let val = game.validate_and_make_move("f1", "b5", ' ');
        if let Err(e) = val {
            panic!("Expected bishop move to be performed, got {:?}", e);
        }

        assert_eq!(game.field[7][5], None);
        assert_eq!(
            game.field[3][1],
            Some({
                ChessPiece {
                    piece: Piece::BISHOP,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "Bb5");
    }

    #[test]
    fn test_bishop_move_with_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[6][4] = None;
        game.field[3][1] = Some(ChessPiece {
            piece: Piece::PAWN,
            color: Color::BLACK,
        });
        let val = game.validate_and_make_move("f1", "b5", ' ');
        if let Err(e) = val {
            panic!("Expected bishop move to be performed, got {:?}", e);
        }

        assert_eq!(game.field[7][5], None);
        assert_eq!(
            game.field[3][1],
            Some({
                ChessPiece {
                    piece: Piece::BISHOP,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "Bxb5");
    }

    #[test]
    fn test_bishop_move_with_wrong_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[6][4] = None;
        game.field[3][1] = Some(ChessPiece {
            piece: Piece::PAWN,
            color: Color::WHITE,
        });
        let val = game.validate_and_make_move("f1", "b5", ' ');
        if val.is_ok() {
            panic!("Expected bishop move to fail due to your own piece being captured");
        }
    }

    #[test]
    fn test_bishop_move_with_piece_in_the_way() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let val = game.validate_and_make_move("f1", "b5", ' ');
        if val.is_ok() {
            panic!("Expected bishop move to fail due to a piece being in the way");
        }
    }
}
