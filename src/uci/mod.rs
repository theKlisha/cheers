pub use crate::{Fen, File, Move, Promotion, Rank, Square};

use crate::uci::{
    de::{deserialize_request, deserialize_response},
    ser::{serialize_request, serialize_response},
};

pub mod de;
pub mod ser;
pub mod stdio;

#[cfg(test)]
mod tests;

impl From<&UciResponse> for String {
    fn from(resp: &UciResponse) -> String {
        serialize_response(resp)
    }
}

impl From<&UciRequest> for String {
    fn from(req: &UciRequest) -> String {
        serialize_request(req)
    }
}

impl TryFrom<&str> for UciRequest {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, String> {
        deserialize_request(s)
            .map(|(_, req)| req)
            .map_err(|e| e.to_string())
    }
}

impl TryFrom<&str> for UciResponse {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, String> {
        deserialize_response(s)
            .map(|(_, resp)| resp)
            .map_err(|e| e.to_string())
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterCommand {
    Later,
    Credentials { name: String, code: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositionSpec {
    StartPos,
    Fen(Fen),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeControl {
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchLimit {
    Infinite,
    Depth(u64),
    Nodes(u64),
    Mate(u64),
    MoveTime(u64),
    TimeControl(TimeControl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoParams {
    pub searchmoves: Vec<Move>,
    pub ponder: bool,
    pub limit: SearchLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UciRequest {
    Uci,
    Debug(bool),
    IsReady,
    SetOption {
        name: String,
        value: Option<String>,
    },
    Register(RegisterCommand),
    UciNewGame,
    Position {
        start: PositionSpec,
        moves: Vec<Move>,
    },
    Go(GoParams),
    Stop,
    PonderHit,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreBound {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Score {
    Centipawns { value: i32, bound: ScoreBound },
    Mate { moves: i32, bound: ScoreBound },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refutation {
    pub mov: Move,
    pub line: Vec<Move>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrLine {
    pub cpu: Option<u64>,
    pub moves: Vec<Move>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InfoFields {
    pub depth: Option<u64>,
    pub seldepth: Option<u64>,
    pub time: Option<u64>,
    pub nodes: Option<u64>,
    pub pv: Option<Vec<Move>>,
    pub multipv: Option<u64>,
    pub score: Option<Score>,
    pub currmove: Option<Move>,
    pub currmovenumber: Option<u64>,
    pub hashfull: Option<u64>,
    pub nps: Option<u64>,
    pub tbhits: Option<u64>,
    pub sbhits: Option<u64>,
    pub cpuload: Option<u64>,
    pub string: Option<String>,
    pub refutation: Option<Refutation>,
    pub currline: Option<CurrLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Checking,
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionType {
    Check { default: bool },
    Spin { default: i64, min: i64, max: i64 },
    Combo { default: String, vars: Vec<String> },
    Button,
    Str { default: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UciResponse {
    IdName(String),
    IdAuthor(String),
    UciOk,
    ReadyOk,
    BestMove {
        mov: Move,
        ponder: Option<Move>,
    },
    CopyProtection(CheckStatus),
    Registration(CheckStatus),
    Info(InfoFields),
    Option {
        name: String,
        option_type: OptionType,
    },
}
