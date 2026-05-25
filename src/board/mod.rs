pub mod bitboard;
pub mod mailbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Pawn,
    Rook,
    Kingt,
    Bishop,
    Queen,
    King,
}

pub type Piece = (Color, Kind);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promotion: Option<Kind>,
}

pub trait Board: Clone {
    fn startpos() -> Self;
    fn from_fen(fen: &str) -> Result<Self, String>;
    fn to_fen(&self) -> String;
    fn side_to_move(&self) -> Color;
    fn piece_at(&self, sq: u8) -> Option<Piece>;
    fn make_move(&mut self, mov: Move);
    fn generate_moves(&self) -> Vec<Move>;
    fn is_in_check(&self, color: Color) -> bool;
}
