#![allow(unused)]

use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, SendError, Sender, channel};

use crate::board::{Board, Color, Move, Piece};
use crate::uci::stdio::StdioUci;
use crate::uci::{File, InfoFields, Rank, Score, ScoreBound, Square, UciEngine, UciHost, UciMove, UciRequest, UciResponse, connect};

pub mod board;
pub mod uci;

pub trait Search {}
pub trait Eval {}

#[derive(Default, Clone)]
pub struct Nil;

impl Board for Nil {
    fn startpos() -> Self { Nil }
    fn from_fen(_: &str) -> Result<Self, String> { Ok(Nil) }
    fn to_fen(&self) -> String { String::new() }
    fn side_to_move(&self) -> Color { Color::White }
    fn piece_at(&self, _: u8) -> Option<Piece> { None }
    fn make_move(&mut self, _: Move) {}
    fn generate_moves(&self) -> Vec<Move> { vec![] }
    fn is_in_check(&self, _: Color) -> bool { false }
}

impl Search for Nil {}
impl Eval for Nil {}

#[derive(Default)]
pub struct Engine<B, S, E>
where
    B: Board,
    S: Search,
    E: Eval,
{
    _board: PhantomData<B>,
    _search: PhantomData<S>,
    _eval: PhantomData<E>,
}

impl<B, S, E> Engine<B, S, E>
where
    B: Board,
    S: Search,
    E: Eval,
{
}

impl<B, S, E> UciEngine for Engine<B, S, E>
where
    B: Board + Send + 'static,
    S: Search + Send + 'static,
    E: Eval + Send + 'static,
{
    fn start(self) -> (Sender<UciRequest>, Receiver<UciResponse>) {
        let (resp_tx, resp_rx) = channel::<UciResponse>();
        let (req_tx, req_rx) = channel::<UciRequest>();

        let _ = std::thread::spawn(move || -> Result<(), SendError<UciResponse>> {
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
                    UciRequest::Go(_) => {
                        resp_tx.send(UciResponse::Info(InfoFields {
                            depth: Some(1),
                            score: Some(Score::Centipawns { value: 0, bound: ScoreBound::Exact }),
                            ..InfoFields::default()
                        }))?;
                        resp_tx.send(UciResponse::BestMove {
                            mov: UciMove {
                                from: Square { file: File::E, rank: Rank::R2 },
                                to: Square { file: File::E, rank: Rank::R4 },
                                promotion: None,
                            },
                            ponder: None,
                        })?;
                    }
                    UciRequest::Quit => return Ok(()),
                    _ => continue,
                }
            }

            Ok(())
        });

        return (req_tx, resp_rx);
    }
}

fn main() {
    let engine = Engine::<Nil, Nil, Nil>::default();
    let host = StdioUci;

    connect(host, engine);
}
