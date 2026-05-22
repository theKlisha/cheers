pub mod bitboard;
pub mod mailbox;

pub enum Color {
    White,
    Black,
}

pub enum Kind {
    Pawn,
    Rook,
    Kingt,
    Bishop,
    Queen,
    King,
}

pub type Piece = (Color, Kind);
