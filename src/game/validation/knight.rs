use crate::{
    game::Game,
    utils::error::{CAPTURE_OWN_PIECE_ERROR, GENERAL_ERROR},
};

pub fn validate_knight_move(
    from: (usize, usize),
    to: (usize, usize),
    game: &Game,
) -> Result<(), &'static str> {
    let row_diff = (from.0 as i32 - to.0 as i32).abs();
    let col_diff = (from.1 as i32 - to.1 as i32).abs();

    // not even move
    match (row_diff, col_diff) {
        (1, 2) | (2, 1) => (),
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
mod test_knight {
    use uuid::Uuid;

    use crate::{
        game::chess_piece::{Color, Piece},
        game::ChessPiece,
        game::Game,
    };

    #[test]
    fn test_knight_move() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let val = game.validate_and_make_move("b1", "c3", ' ');
        if let Err(e) = val {
            panic!("Expected knight move to be performed, got {:?}", e);
        }

        assert_eq!(game.field[7][1], None);
        assert_eq!(
            game.field[5][2],
            Some({
                ChessPiece {
                    piece: Piece::KNIGHT,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "Nc3");
    }

    #[test]
    fn test_knight_move_with_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[6][3] = Some(ChessPiece {
            piece: Piece::ROOK,
            color: Color::BLACK,
        });
        let val = game.validate_and_make_move("b1", "d2", ' ');
        if let Err(e) = val {
            panic!("Expected knight move to be performed, got {:?}", e);
        }

        assert_eq!(game.field[7][1], None);
        assert_eq!(
            game.field[6][3],
            Some({
                ChessPiece {
                    piece: Piece::KNIGHT,
                    color: Color::WHITE,
                }
            })
        );
        assert_eq!(game.previous_move, "Nxd2");
    }

    #[test]
    fn test_knight_move_with_wrong_capture() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        let val = game.validate_and_make_move("b1", "d2", ' ');
        if val.is_ok() {
            panic!("Expected knight move to fail due to your own piece being captured");
        }
    }
}
