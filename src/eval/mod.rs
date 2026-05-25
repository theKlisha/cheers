pub mod static_eval;

use crate::board::Board;

pub trait Eval {
    fn evaluate<B: Board>(&self, board: &B) -> i32;
}
