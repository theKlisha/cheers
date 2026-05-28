pub mod board;
pub mod eval;
pub mod fen;
pub mod search;
pub mod uci;

pub use fen::Fen;

use std::sync::mpsc::{Receiver, Sender};

use crate::uci::{UciRequest, UciResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub type Piece = (Color, Kind);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl From<Promotion> for Kind {
    fn from(p: Promotion) -> Kind {
        match p {
            Promotion::Queen => Kind::Queen,
            Promotion::Rook => Kind::Rook,
            Promotion::Bishop => Kind::Bishop,
            Promotion::Knight => Kind::Knight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub file: File,
    pub rank: Rank,
}

impl From<(File, Rank)> for Square {
    fn from((file, rank): (File, Rank)) -> Self {
        Square { file, rank }
    }
}

impl From<u8> for Square {
    fn from(n: u8) -> Self {
        const FILES: [File; 8] = [
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
        ];
        const RANKS: [Rank; 8] = [
            Rank::R1,
            Rank::R2,
            Rank::R3,
            Rank::R4,
            Rank::R5,
            Rank::R6,
            Rank::R7,
            Rank::R8,
        ];
        Square {
            file: FILES[(n % 8) as usize],
            rank: RANKS[(n / 8) as usize],
        }
    }
}

impl From<Square> for u8 {
    fn from(s: Square) -> u8 {
        s.rank as u8 * 8 + s.file as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn none() -> Self {
        CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    pub fn all() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>,
}

pub trait Board: Clone + From<Fen> + Into<Fen> {
    fn from_fen(s: &str) -> Result<Self, String>
    where
        Self: Sized;
    fn startpos() -> Self
    where
        Self: Sized,
    {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("startpos is always valid")
    }
    fn move_iter(&self) -> impl Iterator<Item = Move>;
    fn piece_iter(&self) -> impl Iterator<Item = Piece>;
    fn piece_at(&self, sq: impl Into<Square>) -> Option<Piece>;
    fn turn(&self) -> Color;
    fn check(&self) -> Option<Color>;
    fn mate(&self) -> Option<Color>;
    fn do_move(&self, mov: &Move) -> Self;
}

pub trait Eval {
    fn evaluate(&self, board: &impl Board) -> i32;
}

pub trait Search {
    fn search(&mut self, board: &impl Board, eval: &impl Eval) -> Option<Move>;
}

pub trait UciHost: Send + 'static {
    fn start(self) -> (Sender<UciResponse>, Receiver<UciRequest>);
}

pub trait UciEngine: Send + 'static {
    fn start(self) -> (Sender<UciRequest>, Receiver<UciResponse>);
}

pub fn connect(host: impl UciHost, engine: impl UciEngine) {
    let (resp_tx, req_rx) = host.start();
    let (req_tx, resp_rx) = engine.start();

    std::thread::spawn(move || {
        for req in req_rx {
            if req_tx.send(req).is_err() {
                break;
            }
        }
    });

    for resp in resp_rx {
        if resp_tx.send(resp).is_err() {
            break;
        }
    }
}
