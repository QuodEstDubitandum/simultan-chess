use crate::{
    game::{chess_piece::Color, Game},
    utils::error::{CAPTURE_OWN_PIECE_ERROR, GENERAL_ERROR, INVALID_CASTLE_ERROR},
};

use super::check_mate::can_be_captured_by;

pub fn validate_king_move(
    from: (usize, usize),
    to: (usize, usize),
    game: &Game,
) -> Result<(), &'static str> {
    let row_diff = (from.0 as i32 - to.0 as i32).abs();
    let col_diff = (from.1 as i32 - to.1 as i32).abs();

    match (row_diff, col_diff) {
        (1, 0) | (0, 1) | (1, 1) => {
            if let Some(piece) = game.field[to.0][to.1] {
                if piece.color == game.next_to_move {
                    return Err(CAPTURE_OWN_PIECE_ERROR);
                }
            }
        }
        (0, 2) => match game.next_to_move {
            Color::WHITE => {
                if game.can_castle.white_can_long_castle
                    && game.field[7][1] == None
                    && game.field[7][2] == None
                    && game.field[7][3] == None
                    && can_be_captured_by(Color::BLACK, (7, 2), game).len() == 0
                    && can_be_captured_by(Color::BLACK, (7, 3), game).len() == 0
                    && can_be_captured_by(Color::BLACK, (7, 4), game).len() == 0
                {
                    return Ok(());
                }
                if game.can_castle.white_can_short_castle
                    && game.field[7][5] == None
                    && game.field[7][6] == None
                {
                    return Ok(());
                }
                return Err(INVALID_CASTLE_ERROR);
            }
            Color::BLACK => {
                if game.can_castle.black_can_long_castle
                    && game.field[0][1] == None
                    && game.field[0][2] == None
                    && game.field[0][3] == None
                    && can_be_captured_by(Color::WHITE, (0, 2), game).len() == 0
                    && can_be_captured_by(Color::WHITE, (0, 3), game).len() == 0
                    && can_be_captured_by(Color::WHITE, (0, 4), game).len() == 0
                {
                    return Ok(());
                }
                if game.can_castle.black_can_short_castle
                    && game.field[0][5] == None
                    && game.field[0][6] == None
                {
                    return Ok(());
                }
                return Err(INVALID_CASTLE_ERROR);
            }
        },
        _ => return Err(GENERAL_ERROR),
    }

    Ok(())
}

#[cfg(test)]
mod test_king {
    use uuid::Uuid;

    use crate::{
        game::chess_piece::{Color, Piece},
        game::{CastlingRights, ChessPiece},
    };

    use crate::game::Game;

    #[test]
    fn test_castle_move() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[7][3] = None;
        game.field[7][2] = None;
        game.field[7][1] = None;
        let val = game.validate_and_make_move("e1", "c1", ' ');
        if let Err(e) = val {
            panic!("Expected castle move to be performed, got {:?}", e);
        }

        assert_eq!(
            game.field[7][2],
            Some(ChessPiece {
                color: Color::WHITE,
                piece: Piece::KING
            })
        );
        assert_eq!(
            game.field[7][3],
            Some(ChessPiece {
                color: Color::WHITE,
                piece: Piece::ROOK
            })
        );

        assert_eq!(game.previous_move, "0-0-0");
    }

    #[test]
    fn test_castle_move_with_piece_in_the_way() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[7][5] = None;
        let val = game.validate_and_make_move("e1", "g1", ' ');
        if val.is_ok() {
            panic!("Expected castle move to fail due to a piece being in the way");
        }
    }

    #[test]
    fn test_castle_move_with_pieces_on_wrong_positions() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[0][5] = None;
        game.field[0][6] = None;
        game.field[0][7] = None;
        let val = game.validate_and_make_move("e8", "g8", ' ');
        if val.is_ok() {
            panic!("Expected castle move to fail due to pieces being on the wrong position");
        }
    }

    #[test]
    fn test_castle_move_after_piece_movement() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.can_castle = CastlingRights {
            white_can_long_castle: true,
            white_can_short_castle: false,
            black_can_short_castle: true,
            black_can_long_castle: true,
        };
        game.field[0][5] = None;
        game.field[0][6] = None;
        let val = game.validate_and_make_move("e8", "g8", ' ');
        if val.is_ok() {
            panic!("Expected castle move to fail due to pieces having already moved before");
        }
    }

    #[test]
    fn test_castle_move_through_check() {
        let mut game = Game::new(Uuid::new_v4(), Color::WHITE);
        game.field[7][3] = None;
        game.field[7][2] = None;
        game.field[7][1] = None;
        game.field[6][3] = None;
        game.field[1][3] = None;
        let val = game.validate_and_make_move("e1", "c1", ' ');
        if val.is_ok() {
            panic!("Expected castle move to fail due to castling through check");
        }
    }
}
