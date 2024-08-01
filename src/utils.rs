pub mod convert_notation;
pub mod error;
pub mod middleware;
pub mod request;
pub mod response;

pub fn is_in_bounds(row: i32, col: i32) -> bool {
    row >= 0 && row <= 7 && col >= 0 && col <= 7
}
