#![allow(unused)]

use std::sync::mpsc::{Receiver, SendError, Sender, channel};

use crate::board::mailbox::Mailbox;
use crate::board::{Board, Kind, Move};
use crate::eval::Eval;
use crate::eval::static_eval::StaticEval;
use crate::search::Search;
use crate::search::random::RandomSearch;
use crate::uci::stdio::StdioUci;
use crate::uci::{
    File, InfoFields, PositionSpec, Promotion, Rank, Score, ScoreBound, Square, UciEngine, UciHost,
    UciMove, UciRequest, UciResponse, connect,
};

pub mod board;
pub mod eval;
pub mod search;
pub mod uci;

fn file_to_u8(f: File) -> u8 {
    match f {
        File::A => 0,
        File::B => 1,
        File::C => 2,
        File::D => 3,
        File::E => 4,
        File::F => 5,
        File::G => 6,
        File::H => 7,
    }
}

fn u8_to_file(n: u8) -> File {
    [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ][n as usize]
}

fn rank_to_u8(r: Rank) -> u8 {
    match r {
        Rank::R1 => 0,
        Rank::R2 => 1,
        Rank::R3 => 2,
        Rank::R4 => 3,
        Rank::R5 => 4,
        Rank::R6 => 5,
        Rank::R7 => 6,
        Rank::R8 => 7,
    }
}

fn u8_to_rank(n: u8) -> Rank {
    [
        Rank::R1,
        Rank::R2,
        Rank::R3,
        Rank::R4,
        Rank::R5,
        Rank::R6,
        Rank::R7,
        Rank::R8,
    ][n as usize]
}

fn uci_to_board_move(m: &UciMove) -> Move {
    Move {
        from: rank_to_u8(m.from.rank) * 8 + file_to_u8(m.from.file),
        to: rank_to_u8(m.to.rank) * 8 + file_to_u8(m.to.file),
        promotion: m.promotion.map(|p| match p {
            Promotion::Queen => Kind::Queen,
            Promotion::Rook => Kind::Rook,
            Promotion::Bishop => Kind::Bishop,
            Promotion::Knight => Kind::Kingt,
        }),
    }
}

fn board_to_uci_move(m: &Move) -> UciMove {
    UciMove {
        from: Square {
            file: u8_to_file(m.from % 8),
            rank: u8_to_rank(m.from / 8),
        },
        to: Square {
            file: u8_to_file(m.to % 8),
            rank: u8_to_rank(m.to / 8),
        },
        promotion: m.promotion.map(|k| match k {
            Kind::Queen => Promotion::Queen,
            Kind::Rook => Promotion::Rook,
            Kind::Bishop => Promotion::Bishop,
            Kind::Kingt => Promotion::Knight,
            _ => unreachable!(),
        }),
    }
}

pub struct Engine<B, S, E>
where
    B: Board,
    S: Search,
    E: Eval,
{
    _board: std::marker::PhantomData<B>,
    _search: std::marker::PhantomData<S>,
    _eval: std::marker::PhantomData<E>,
}

impl<B, S, E> Default for Engine<B, S, E>
where
    B: Board,
    S: Search,
    E: Eval,
{
    fn default() -> Self {
        Engine {
            _board: std::marker::PhantomData,
            _search: std::marker::PhantomData,
            _eval: std::marker::PhantomData,
        }
    }
}

impl<B, S, E> UciEngine for Engine<B, S, E>
where
    B: Board + Send + 'static,
    S: Search + Default + Send + 'static,
    E: Eval + Default + Send + 'static,
{
    fn start(self) -> (Sender<UciRequest>, Receiver<UciResponse>) {
        let (resp_tx, resp_rx) = channel::<UciResponse>();
        let (req_tx, req_rx) = channel::<UciRequest>();

        let _ = std::thread::spawn(move || -> Result<(), SendError<UciResponse>> {
            let mut board = B::startpos();
            let mut search = S::default();
            let eval = E::default();

            for req in req_rx {
                match req {
                    UciRequest::Uci => {
                        resp_tx.send(UciResponse::IdName("cheers".to_string()))?;
                        resp_tx.send(UciResponse::IdAuthor("theklisha".to_string()))?;
                        resp_tx.send(UciResponse::UciOk)?;
                    }
                    UciRequest::IsReady => {
                        resp_tx.send(UciResponse::ReadyOk)?;
                    }
                    UciRequest::UciNewGame => {
                        board = B::startpos();
                    }
                    UciRequest::Position { start, moves } => {
                        board = match start {
                            PositionSpec::StartPos => B::startpos(),
                            PositionSpec::Fen(fen) => {
                                B::from_fen(&fen).unwrap_or_else(|_| B::startpos())
                            }
                        };
                        for uci_mov in &moves {
                            board.make_move(uci_to_board_move(uci_mov));
                        }
                    }
                    UciRequest::Go(_) => {
                        let eval_score = eval.evaluate(&board);
                        resp_tx.send(UciResponse::Info(InfoFields {
                            depth: Some(1),
                            score: Some(Score::Centipawns {
                                value: eval_score,
                                bound: ScoreBound::Exact,
                            }),
                            ..InfoFields::default()
                        }))?;
                        if let Some(m) = search.search(&board, &eval) {
                            resp_tx.send(UciResponse::BestMove {
                                mov: board_to_uci_move(&m),
                                ponder: None,
                            })?;
                        }
                    }
                    UciRequest::Quit => return Ok(()),
                    _ => continue,
                }
            }

            Ok(())
        });

        (req_tx, resp_rx)
    }
}

fn main() {
    let engine = Engine::<Mailbox, RandomSearch, StaticEval>::default();
    let host = StdioUci;
    connect(host, engine);
}
