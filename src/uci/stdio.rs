use std::io::{self, BufRead, Write};
use std::sync::mpsc::{Receiver, Sender, channel};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{one_of, space1, u64 as parse_u64},
    combinator::{map, opt, rest, value},
    multi::{many0, separated_list1},
    sequence::{pair, preceded},
};

use super::{
    CheckStatus, CurrLine, File, GoParams, InfoFields, OptionType, PositionSpec, Promotion, Rank,
    Refutation, RegisterCommand, Score, ScoreBound, SearchLimit, Square, TimeControl, UciHost,
    UciMove, UciRequest, UciResponse,
};

pub struct StdioUci;

impl UciHost for StdioUci {
    fn start(self) -> (Sender<UciResponse>, Receiver<UciRequest>) {
        let (req_tx, req_rx) = channel();
        let (resp_tx, resp_rx) = channel();

        std::thread::spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let Ok(line) = line else { break };
                if let Some(req) = parse_request(&line) {
                    if req_tx.send(req).is_err() {
                        break;
                    }
                }
            }
        });

        std::thread::spawn(move || {
            let stdout = io::stdout();
            for resp in resp_rx {
                let mut out = stdout.lock();
                if writeln!(out, "{}", format_response(resp)).is_err() {
                    break;
                }
                let _ = out.flush();
            }
        });

        (resp_tx, req_rx)
    }
}

// Parsing

pub fn parse_request(line: &str) -> Option<UciRequest> {
    command(line.trim()).map(|(_, req)| req).ok()
}

fn command(i: &str) -> IResult<&str, UciRequest> {
    alt((
        value(UciRequest::Uci, tag("uci")),
        value(UciRequest::IsReady, tag("isready")),
        value(UciRequest::UciNewGame, tag("ucinewgame")),
        value(UciRequest::Stop, tag("stop")),
        value(UciRequest::PonderHit, tag("ponderhit")),
        value(UciRequest::Quit, tag("quit")),
        parse_debug,
        parse_setoption,
        parse_register,
        parse_position,
        parse_go,
    ))
    .parse(i)
}

fn parse_square(i: &str) -> IResult<&str, Square> {
    let (i, file) = map(one_of("abcdefgh"), |c| match c {
        'a' => File::A,
        'b' => File::B,
        'c' => File::C,
        'd' => File::D,
        'e' => File::E,
        'f' => File::F,
        'g' => File::G,
        'h' => File::H,
        _ => unreachable!(),
    })
    .parse(i)?;
    let (i, rank) = map(one_of("12345678"), |c| match c {
        '1' => Rank::R1,
        '2' => Rank::R2,
        '3' => Rank::R3,
        '4' => Rank::R4,
        '5' => Rank::R5,
        '6' => Rank::R6,
        '7' => Rank::R7,
        '8' => Rank::R8,
        _ => unreachable!(),
    })
    .parse(i)?;
    Ok((i, Square { file, rank }))
}

fn parse_move(i: &str) -> IResult<&str, UciMove> {
    let (i, from) = parse_square(i)?;
    let (i, to) = parse_square(i)?;
    let (i, promotion) = opt(map(one_of("qrbn"), |c| match c {
        'q' => Promotion::Queen,
        'r' => Promotion::Rook,
        'b' => Promotion::Bishop,
        'n' => Promotion::Knight,
        _ => unreachable!(),
    }))
    .parse(i)?;
    Ok((i, UciMove { from, to, promotion }))
}

fn parse_debug(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("debug")(i)?;
    let (i, _) = space1(i)?;
    alt((
        value(UciRequest::Debug(true), tag("on")),
        value(UciRequest::Debug(false), tag("off")),
    ))
    .parse(i)
}

fn parse_setoption(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("setoption")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("name")(i)?;
    let (i, _) = space1(i)?;
    let (i, name) = alt((take_until(" value"), rest)).parse(i)?;
    let (i, val) = opt(preceded(
        pair(space1, tag("value")),
        opt(preceded(space1, rest)),
    ))
    .parse(i)?;
    Ok((i, UciRequest::SetOption {
        name: name.trim_end().to_string(),
        value: val.flatten().map(str::to_string),
    }))
}

fn parse_register(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("register")(i)?;
    let (i, _) = space1(i)?;
    alt((
        value(UciRequest::Register(RegisterCommand::Later), tag("later")),
        parse_register_credentials,
    ))
    .parse(i)
}

fn parse_register_credentials(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("name")(i)?;
    let (i, _) = space1(i)?;
    let (i, name) = take_until(" code")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("code")(i)?;
    let (i, _) = space1(i)?;
    let (i, code) = rest(i)?;
    Ok((i, UciRequest::Register(RegisterCommand::Credentials {
        name: name.to_string(),
        code: code.to_string(),
    })))
}

fn parse_fen_spec(i: &str) -> IResult<&str, PositionSpec> {
    let (i, _) = tag("fen")(i)?;
    let (i, _) = space1(i)?;
    let (i, fen) = alt((take_until(" moves"), rest)).parse(i)?;
    Ok((i, PositionSpec::Fen(fen.to_string())))
}

fn parse_position(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("position")(i)?;
    let (i, _) = space1(i)?;
    let (i, start) = alt((
        value(PositionSpec::StartPos, tag("startpos")),
        parse_fen_spec,
    ))
    .parse(i)?;
    let (i, moves) = opt(preceded(
        pair(space1, tag("moves")),
        many0(preceded(space1, parse_move)),
    ))
    .parse(i)?;
    Ok((i, UciRequest::Position {
        start,
        moves: moves.unwrap_or_default(),
    }))
}

#[derive(Clone)]
enum GoParam {
    Searchmoves(Vec<UciMove>),
    Ponder,
    Wtime(u64),
    Btime(u64),
    Winc(u64),
    Binc(u64),
    Movestogo(u64),
    Depth(u64),
    Nodes(u64),
    Mate(u64),
    Movetime(u64),
    Infinite,
}

fn parse_go_param(i: &str) -> IResult<&str, GoParam> {
    alt((
        value(GoParam::Infinite, tag("infinite")),
        value(GoParam::Ponder, tag("ponder")),
        map(preceded(pair(tag("wtime"), space1), parse_u64), GoParam::Wtime),
        map(preceded(pair(tag("btime"), space1), parse_u64), GoParam::Btime),
        map(preceded(pair(tag("winc"), space1), parse_u64), GoParam::Winc),
        map(preceded(pair(tag("binc"), space1), parse_u64), GoParam::Binc),
        map(preceded(pair(tag("movestogo"), space1), parse_u64), GoParam::Movestogo),
        map(preceded(pair(tag("depth"), space1), parse_u64), GoParam::Depth),
        map(preceded(pair(tag("nodes"), space1), parse_u64), GoParam::Nodes),
        map(preceded(pair(tag("mate"), space1), parse_u64), GoParam::Mate),
        map(preceded(pair(tag("movetime"), space1), parse_u64), GoParam::Movetime),
        map(
            preceded(pair(tag("searchmoves"), space1), separated_list1(space1, parse_move)),
            GoParam::Searchmoves,
        ),
    ))
    .parse(i)
}

fn parse_go(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("go")(i)?;
    let (i, params) = many0(preceded(space1, parse_go_param)).parse(i)?;

    let mut searchmoves = vec![];
    let mut ponder = false;
    let mut wtime = None;
    let mut btime = None;
    let mut winc = None;
    let mut binc = None;
    let mut movestogo = None;
    let mut depth = None;
    let mut nodes = None;
    let mut mate = None;
    let mut movetime = None;
    let mut infinite = false;

    for param in params {
        match param {
            GoParam::Searchmoves(m) => searchmoves = m,
            GoParam::Ponder => ponder = true,
            GoParam::Wtime(v) => wtime = Some(v),
            GoParam::Btime(v) => btime = Some(v),
            GoParam::Winc(v) => winc = Some(v),
            GoParam::Binc(v) => binc = Some(v),
            GoParam::Movestogo(v) => movestogo = Some(v),
            GoParam::Depth(v) => depth = Some(v),
            GoParam::Nodes(v) => nodes = Some(v),
            GoParam::Mate(v) => mate = Some(v),
            GoParam::Movetime(v) => movetime = Some(v),
            GoParam::Infinite => infinite = true,
        }
    }

    let limit = if infinite {
        SearchLimit::Infinite
    } else if let Some(d) = depth {
        SearchLimit::Depth(d)
    } else if let Some(n) = nodes {
        SearchLimit::Nodes(n)
    } else if let Some(m) = mate {
        SearchLimit::Mate(m)
    } else if let Some(mt) = movetime {
        SearchLimit::MoveTime(mt)
    } else {
        SearchLimit::TimeControl(TimeControl { wtime, btime, winc, binc, movestogo })
    };

    Ok((i, UciRequest::Go(GoParams { searchmoves, ponder, limit })))
}

// Serialization

pub fn format_response(resp: UciResponse) -> String {
    match resp {
        UciResponse::IdName(name) => format!("id name {name}"),
        UciResponse::IdAuthor(author) => format!("id author {author}"),
        UciResponse::UciOk => "uciok".to_string(),
        UciResponse::ReadyOk => "readyok".to_string(),
        UciResponse::BestMove { mov, ponder } => {
            let mut s = format!("bestmove {}", fmt_move(mov));
            if let Some(p) = ponder {
                s.push_str(&format!(" ponder {}", fmt_move(p)));
            }
            s
        },
        UciResponse::CopyProtection(status) => {
            format!("copyprotection {}", fmt_check_status(&status))
        },
        UciResponse::Registration(status) => {
            format!("registration {}", fmt_check_status(&status))
        },
        UciResponse::Info(f) => fmt_info(f),
        UciResponse::Option { name, option_type } => fmt_option(&name, &option_type),
    }
}

fn fmt_move(m: UciMove) -> String {
    let file = |f| match f {
        File::A => 'a',
        File::B => 'b',
        File::C => 'c',
        File::D => 'd',
        File::E => 'e',
        File::F => 'f',
        File::G => 'g',
        File::H => 'h',
    };
    let rank = |r| match r {
        Rank::R1 => '1',
        Rank::R2 => '2',
        Rank::R3 => '3',
        Rank::R4 => '4',
        Rank::R5 => '5',
        Rank::R6 => '6',
        Rank::R7 => '7',
        Rank::R8 => '8',
    };
    let mut s = format!(
        "{}{}{}{}",
        file(m.from.file),
        rank(m.from.rank),
        file(m.to.file),
        rank(m.to.rank)
    );
    if let Some(p) = m.promotion {
        s.push(match p {
            Promotion::Queen => 'q',
            Promotion::Rook => 'r',
            Promotion::Bishop => 'b',
            Promotion::Knight => 'n',
        });
    }
    s
}

fn fmt_moves(moves: &[UciMove]) -> String {
    moves.iter().map(|&m| fmt_move(m)).collect::<Vec<_>>().join(" ")
}

fn fmt_check_status(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Checking => "checking",
        CheckStatus::Ok => "ok",
        CheckStatus::Error => "error",
    }
}

fn fmt_bound(b: &ScoreBound) -> &'static str {
    match b {
        ScoreBound::Exact => "",
        ScoreBound::LowerBound => " lowerbound",
        ScoreBound::UpperBound => " upperbound",
    }
}

fn fmt_info(f: InfoFields) -> String {
    macro_rules! field {
        ($out:expr, $name:literal, $val:expr) => {
            if let Some(v) = $val {
                $out.push_str(&format!(concat!(" ", $name, " {}"), v));
            }
        };
    }

    let mut out = String::from("info");
    field!(out, "depth", f.depth);
    field!(out, "seldepth", f.seldepth);
    field!(out, "time", f.time);
    field!(out, "nodes", f.nodes);
    if let Some(pv) = f.pv {
        out.push_str(&format!(" pv {}", fmt_moves(&pv)));
    }
    field!(out, "multipv", f.multipv);
    if let Some(score) = f.score {
        match score {
            Score::Centipawns { value, bound } => {
                out.push_str(&format!(" score cp {value}{}", fmt_bound(&bound)));
            },
            Score::Mate { moves, bound } => {
                out.push_str(&format!(" score mate {moves}{}", fmt_bound(&bound)));
            },
        }
    }
    if let Some(m) = f.currmove {
        out.push_str(&format!(" currmove {}", fmt_move(m)));
    }
    field!(out, "currmovenumber", f.currmovenumber);
    field!(out, "hashfull", f.hashfull);
    field!(out, "nps", f.nps);
    field!(out, "tbhits", f.tbhits);
    field!(out, "sbhits", f.sbhits);
    field!(out, "cpuload", f.cpuload);
    if let Some(s) = f.string {
        out.push_str(&format!(" string {s}"));
    }
    if let Some(Refutation { mov, line }) = f.refutation {
        out.push_str(&format!(" refutation {}", fmt_move(mov)));
        if !line.is_empty() {
            out.push_str(&format!(" {}", fmt_moves(&line)));
        }
    }
    if let Some(CurrLine { cpu, moves }) = f.currline {
        out.push_str(" currline");
        if let Some(cpu) = cpu {
            out.push_str(&format!(" {cpu}"));
        }
        if !moves.is_empty() {
            out.push_str(&format!(" {}", fmt_moves(&moves)));
        }
    }
    out
}

fn fmt_option(name: &str, opt: &OptionType) -> String {
    match opt {
        OptionType::Check { default } => {
            format!("option name {name} type check default {default}")
        },
        OptionType::Spin { default, min, max } => {
            format!("option name {name} type spin default {default} min {min} max {max}")
        },
        OptionType::Combo { default, vars } => {
            let vars: String = vars.iter().map(|v| format!(" var {v}")).collect();
            format!("option name {name} type combo default {default}{vars}")
        },
        OptionType::Button => format!("option name {name} type button"),
        OptionType::Str { default } => {
            let d = default.as_deref().unwrap_or("<empty>");
            format!("option name {name} type string default {d}")
        },
    }
}
