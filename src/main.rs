#![allow(unused)]

use std::sync::mpsc::{Receiver, SendError, Sender, channel};

use cheers::board::mailbox::Mailbox;
use cheers::eval::static_eval::StaticEval;
use cheers::search::random::RandomSearch;
use cheers::uci::stdio::StdioUci;
use cheers::uci::{InfoFields, PositionSpec, Score, ScoreBound, UciRequest, UciResponse};
use cheers::{Board, Eval, Move, Search, UciEngine, UciHost, connect};

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
                            PositionSpec::Fen(fen) => B::from_fen(&fen),
                        };
                        for uci_mov in moves {
                            board = board.do_move(uci_mov);
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
                                mov: m,
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
