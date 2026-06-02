use std::slice::Iter;

use crate::Board;

#[derive(Clone, Copy)]
struct Bb(u64);

#[derive(Clone)]
struct Occupancy {
    pawn: Bb,
    rook: Bb,
    kingt: Bb,
    bishop: Bb,
    queen: Bb,
    king: Bb,
}

#[derive(Clone)]
pub struct Bitboard {
    white_occ: Occupancy,
    black_occ: Occupancy,
}

impl Board for Bitboard {
    fn move_iter(&self) -> impl Iterator<Item = crate::Move> {
        [].into_iter()
    }

    fn piece_iter(&self) -> impl Iterator<Item = crate::Piece> {
        [].into_iter()
    }

    fn piece_at(&self, sq: impl Into<crate::Square>) -> Option<crate::Piece> {
        todo!()
    }

    fn turn(&self) -> crate::Color {
        todo!()
    }

    fn check(&self) -> Option<crate::Color> {
        todo!()
    }

    fn mate(&self) -> Option<crate::Color> {
        todo!()
    }

    fn do_move(&self, mov: impl Into<crate::Move>) -> Self {
        todo!()
    }

    fn fen(&self) -> crate::Fen {
        todo!()
    }

    fn from_fen(s: &crate::Fen) -> Self {
        todo!()
    }
}
