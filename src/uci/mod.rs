use std::{
    io,
    sync::mpsc::{Receiver, Sender, channel},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    pub file: File,
    pub rank: Rank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UciMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>,
}

pub enum RegisterCommand {
    Later,
    Credentials { name: String, code: String },
}

pub enum PositionSpec {
    StartPos,
    Fen(String),
}

pub struct TimeControl {
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
}

pub enum SearchLimit {
    Infinite,
    Depth(u64),
    Nodes(u64),
    Mate(u64),
    MoveTime(u64),
    TimeControl(TimeControl),
}

pub struct GoParams {
    pub searchmoves: Vec<UciMove>,
    pub ponder: bool,
    pub limit: SearchLimit,
}

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
        moves: Vec<UciMove>,
    },
    Go(GoParams),
    Stop,
    PonderHit,
    Quit,
}

pub enum ScoreBound {
    Exact,
    LowerBound,
    UpperBound,
}

pub enum Score {
    Centipawns { value: i32, bound: ScoreBound },
    Mate { moves: i32, bound: ScoreBound },
}

pub struct Refutation {
    pub mov: UciMove,
    pub line: Vec<UciMove>,
}

pub struct CurrLine {
    pub cpu: Option<u64>,
    pub moves: Vec<UciMove>,
}

pub struct InfoFields {
    pub depth: Option<u64>,
    pub seldepth: Option<u64>,
    pub time: Option<u64>,
    pub nodes: Option<u64>,
    pub pv: Option<Vec<UciMove>>,
    pub multipv: Option<u64>,
    pub score: Option<Score>,
    pub currmove: Option<UciMove>,
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

pub enum CheckStatus {
    Checking,
    Ok,
    Error,
}

pub enum OptionType {
    Check { default: bool },
    Spin { default: i64, min: i64, max: i64 },
    Combo { default: String, vars: Vec<String> },
    Button,
    Str { default: Option<String> },
}

pub enum UciResponse {
    IdName(String),
    IdAuthor(String),
    UciOk,
    ReadyOk,
    BestMove {
        mov: UciMove,
        ponder: Option<UciMove>,
    },
    CopyProtection(CheckStatus),
    Registration(CheckStatus),
    Info(InfoFields),
    Option {
        name: String,
        option_type: OptionType,
    },
}

pub trait UciHost {
    fn read(&self) -> Option<UciRequest>;
    fn write(&self, response: UciResponse) -> io::Result<()>;
}

pub trait UciEngine {
    fn read(&self) -> Option<UciResponse>;
    fn write(&self, request: UciRequest) -> io::Result<()>;
}

pub fn connect(host: impl UciHost, engine: impl UciEngine) -> io::Result<()> {
    Ok(())
}
