use crate::{Board, Eval, Move, Search};

pub struct RandomSearch {
    state: u64,
}

impl Default for RandomSearch {
    fn default() -> Self {
        RandomSearch { state: 1 }
    }
}

impl RandomSearch {
    pub fn new(seed: u64) -> Self {
        RandomSearch {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

impl Search for RandomSearch {
    fn search(&mut self, board: &impl Board, _eval: &impl Eval) -> Option<Move> {
        let moves: Vec<Move> = board.move_iter().collect();
        if moves.is_empty() {
            return None;
        }
        let idx = (self.next_u64() as usize) % moves.len();
        Some(moves[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::mailbox::Mailbox;
    use crate::eval::static_eval::StaticEval;

    #[test]
    fn returns_legal_move_from_startpos() {
        let mut search = RandomSearch::new(12345);
        let eval = StaticEval::new(0);
        let board = Mailbox::startpos();
        let mov = search.search(&board, &eval);
        assert!(mov.is_some());
        let legal_moves = board.generate_moves();
        assert!(legal_moves.contains(&mov.unwrap()));
    }

    #[test]
    fn returns_none_on_checkmate() {
        let mut search = RandomSearch::new(12345);
        let eval = StaticEval::new(0);
        let board =
            Mailbox::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        assert_eq!(search.search(&board, &eval), None);
    }

    #[test]
    fn different_seeds_produce_variety() {
        let eval = StaticEval::new(0);
        let board = Mailbox::startpos();
        let mut results = std::collections::HashSet::new();
        for seed in 1..=100u64 {
            let mut search = RandomSearch::new(seed);
            if let Some(m) = search.search(&board, &eval) {
                results.insert((m.from, m.to));
            }
        }
        assert!(results.len() > 1);
    }

    #[test]
    fn successive_calls_vary() {
        let eval = StaticEval::new(0);
        let board = Mailbox::startpos();
        let mut search = RandomSearch::new(42);
        let mut results = std::collections::HashSet::new();
        for _ in 0..50 {
            if let Some(m) = search.search(&board, &eval) {
                results.insert((m.from, m.to));
            }
        }
        assert!(results.len() > 1);
    }
}
