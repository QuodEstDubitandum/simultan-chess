use crate::{
    game::{
        chess_piece::{ChessPiece, Color, Piece},
        Game,
    },
    utils::{convert_notation::get_notation_from_square, is_in_bounds},
};

#[derive(Debug)]
pub struct CapturePiece {
    pub row: usize,
    pub col: usize,
    pub piece: Piece,
}

pub fn is_mate(game: &Game) -> bool {
    let enemy_color: Color;
    let king_position: (usize, usize);

    match game.next_to_move {
        Color::WHITE => {
            enemy_color = Color::BLACK;
            king_position = game.king_position.white_king_position;
        }
        Color::BLACK => {
            enemy_color = Color::WHITE;
            king_position = game.king_position.black_king_position;
        }
    }

    let threatening_pieces = can_be_captured_by(enemy_color, king_position, game);
    if threatening_pieces.len() == 0 {
        return false;
    }

    let king_row = king_position.0 as i32;
    let king_col = king_position.1 as i32;

    //Check all surrounding pieces
    let surrounding_squares: Vec<(i32, i32)> = vec![
        (king_row + 1, king_col - 1),
        (king_row + 1, king_col),
        (king_row + 1, king_col + 1),
        (king_row, king_col - 1),
        (king_row, king_col + 1),
        (king_row - 1, king_col - 1),
        (king_row - 1, king_col),
        (king_row - 1, king_col + 1),
    ];
    for surr_sq in surrounding_squares {
        if is_in_bounds(surr_sq.0, surr_sq.1)
            && game.field[surr_sq.0 as usize][surr_sq.1 as usize].is_none()
            && can_be_captured_by(enemy_color, (surr_sq.0 as usize, surr_sq.1 as usize), game).len()
                == 0
        {
            return false;
        }
    }

    // if there are more than 2 pieces threatening the king and he cannot move to another
    // square, its mate
    if threatening_pieces.len() > 1 {
        return true;
    }

    // else we need to check if this one threatening piece can be captured to avoid mate
    let saving_pieces = can_be_captured_by(
        game.next_to_move,
        (threatening_pieces[0].row, threatening_pieces[0].col),
        game,
    );

    // and then check if after the "saving move" the king is still in check
    for piece in saving_pieces {
        let algebraic_from = get_notation_from_square((piece.row, piece.col)).unwrap();
        let algebraic_to =
            get_notation_from_square((threatening_pieces[0].row, threatening_pieces[0].col))
                .unwrap();
        println!("{:?} {:?}", algebraic_from, algebraic_to);
        if can_king_be_captured_after_move(game, &algebraic_from, &algebraic_to, 'Q').len() == 0 {
            return false;
        };
    }

    true
}

pub fn can_king_be_captured_after_move(
    game: &Game,
    algebraic_from: &str,
    algebraic_to: &str,
    promotion_ch: char,
) -> Vec<CapturePiece> {
    let mut game_clone = game.clone();
    game_clone.make_move(algebraic_from, algebraic_to, promotion_ch);
    match game_clone.next_to_move {
        Color::BLACK => can_be_captured_by(
            Color::BLACK,
            game_clone.king_position.white_king_position,
            &game_clone,
        ),
        Color::WHITE => can_be_captured_by(
            Color::WHITE,
            game_clone.king_position.black_king_position,
            &game_clone,
        ),
    }
}

pub fn can_be_captured_by(
    enemy_color: Color,
    square: (usize, usize),
    game: &Game,
) -> Vec<CapturePiece> {
    let mut capturable_by = vec![];

    capturable_by_knight(enemy_color, square, game, &mut capturable_by);
    capturable_by_diagonal_move(enemy_color, square, game, &mut capturable_by);
    capturable_by_linear_move(enemy_color, square, game, &mut capturable_by);

    capturable_by
}

fn capturable_by_knight(
    enemy_color: Color,
    square: (usize, usize),
    game: &Game,
    capturable_by: &mut Vec<CapturePiece>,
) {
    let row = square.0 as i32;
    let col = square.1 as i32;

    let knight_squares = vec![
        (row + 2, col + 1),
        (row + 2, col - 1),
        (row + 1, col + 2),
        (row + 1, col - 2),
        (row - 2, col + 1),
        (row - 2, col - 1),
        (row - 1, col + 2),
        (row - 1, col - 2),
    ];

    for (row, col) in knight_squares {
        if !is_in_bounds(row, col) {
            continue;
        }

        if game.field[row as usize][col as usize]
            == Some(ChessPiece {
                piece: Piece::KNIGHT,
                color: enemy_color,
            })
        {
            capturable_by.push(CapturePiece {
                row: row as usize,
                col: col as usize,
                piece: Piece::KNIGHT,
            });
        }
    }
}

fn capturable_by_diagonal_move(
    enemy_color: Color,
    square: (usize, usize),
    game: &Game,
    capturable_by: &mut Vec<CapturePiece>,
) {
    let row = square.0 as i32;
    let col = square.1 as i32;

    let directions = vec![(1, 1), (1, -1), (-1, 1), (-1, -1)];
    'outer: for dir in directions {
        for i in 1..8 {
            if !is_in_bounds(row + i * dir.0, col + i * dir.1) {
                continue 'outer;
            }

            let row = (row + i * dir.0) as usize;
            let col = (col + i * dir.1) as usize;

            if let Some(piece) = game.field[row][col] {
                if piece.color != enemy_color {
                    continue 'outer;
                }

                match piece.piece {
                    Piece::QUEEN => {
                        capturable_by.push(CapturePiece {
                            row,
                            col,
                            piece: Piece::QUEEN,
                        });
                    }
                    Piece::BISHOP => {
                        capturable_by.push(CapturePiece {
                            row,
                            col,
                            piece: Piece::BISHOP,
                        });
                    }
                    Piece::PAWN => {
                        if i == 1 {
                            match (dir.0, dir.1, enemy_color) {
                                (-1, 1, Color::BLACK)
                                | (-1, -1, Color::BLACK)
                                | (1, 1, Color::WHITE)
                                | (1, -1, Color::WHITE) => {
                                    capturable_by.push(CapturePiece {
                                        row,
                                        col,
                                        piece: Piece::PAWN,
                                    });
                                }
                                _ => (),
                            }
                        }
                    }
                    Piece::KING => {
                        if i == 1 {
                            capturable_by.push(CapturePiece {
                                row,
                                col,
                                piece: Piece::KING,
                            });
                        }
                    }
                    _ => (),
                }
                continue 'outer;
            }
        }
    }
}

fn capturable_by_linear_move(
    enemy_color: Color,
    square: (usize, usize),
    game: &Game,
    capturable_by: &mut Vec<CapturePiece>,
) {
    let row = square.0 as i32;
    let col = square.1 as i32;

    let directions = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];
    'outer: for dir in directions {
        for i in 1..8 {
            if !is_in_bounds(row + i * dir.0, col + i * dir.1) {
                continue 'outer;
            }

            let row = (row + i * dir.0) as usize;
            let col = (col + i * dir.1) as usize;

            if let Some(piece) = game.field[row][col] {
                if piece.color != enemy_color {
                    continue 'outer;
                }

                match piece.piece {
                    Piece::QUEEN => {
                        capturable_by.push(CapturePiece {
                            row,
                            col,
                            piece: Piece::QUEEN,
                        });
                    }
                    Piece::ROOK => {
                        capturable_by.push(CapturePiece {
                            row,
                            col,
                            piece: Piece::ROOK,
                        });
                    }
                    Piece::KING => {
                        if i == 1 {
                            capturable_by.push(CapturePiece {
                                row,
                                col,
                                piece: Piece::KING,
                            });
                        }
                    }
                    _ => (),
                }
                continue 'outer;
            }
        }
    }
}
