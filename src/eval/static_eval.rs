use crate::board::Board;
use crate::eval::Eval;

pub struct StaticEval {
    value: i32,
}

impl StaticEval {
    pub fn new(value: i32) -> Self {
        StaticEval { value }
    }
}

impl Default for StaticEval {
    fn default() -> Self {
        StaticEval { value: 0 }
    }
}

impl Eval for StaticEval {
    fn evaluate<B: Board>(&self, _board: &B) -> i32 {
        self.value
    }
}
