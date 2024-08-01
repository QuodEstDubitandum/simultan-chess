pub const GENERAL_ERROR: &'static str = "Invalid move";
pub const SQUARE_OUT_OF_BOUNDS_ERROR: &'static str =
    "The selected square is not inside the bounds of the chessboard";
pub const PIECE_IN_THE_WAY_ERROR: &'static str = "There is a piece in the way of your move";
pub const CAPTURE_OWN_PIECE_ERROR: &'static str = "You cannot capture your own piece";
pub const PROMOTION_ERROR: &'static str = "No promotion piece specified";
pub const NO_PIECE_SELECTED_ERROR: &'static str = "You have not selected any piece";
pub const INVALID_FROM_FIELD: &'static str = "The from field in your requests body is incorrect";
pub const INVALID_TO_FIELD: &'static str = "The to field in your requests body is incorrect";
pub const INVALID_CASTLE_ERROR: &'static str = "That castle move is invalid";
pub const CHECK_ERROR: &'static str = "Your king is in check";
pub const INTERNAL_SERVER_ERROR: &'static str = "Internal server error";
