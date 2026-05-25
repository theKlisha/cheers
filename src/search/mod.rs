pub mod random;

use crate::board::{Board, Move};
use crate::eval::Eval;

pub trait Search {
    fn search<B: Board, E: Eval>(&mut self, board: &B, eval: &E) -> Option<Move>;
}
