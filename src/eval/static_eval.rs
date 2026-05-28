use crate::{Board, Eval};

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
    fn evaluate(&self, _board: &impl Board) -> i32 {
        self.value
    }
}
