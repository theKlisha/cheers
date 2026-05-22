use super::*;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{i64 as parse_i64, one_of, space1, u64 as parse_u64},
    combinator::{map, opt, rest, value},
    multi::{many0, separated_list1},
    sequence::{pair, preceded},
};

impl From<&UciResponse> for String {
    fn from(resp: &UciResponse) -> String {
        fmt_response(resp)
    }
}

impl From<&UciRequest> for String {
    fn from(req: &UciRequest) -> String {
        format_request(req)
    }
}

impl TryFrom<&str> for UciRequest {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, ()> {
        parse_request(s).ok_or(())
    }
}

impl TryFrom<&str> for UciResponse {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, ()> {
        parse_response(s).ok_or(())
    }
}

// Serialization

pub fn fmt_response(resp: &UciResponse) -> String {
    match resp {
        UciResponse::IdName(name) => format!("id name {name}"),
        UciResponse::IdAuthor(author) => format!("id author {author}"),
        UciResponse::UciOk => "uciok".to_string(),
        UciResponse::ReadyOk => "readyok".to_string(),
        UciResponse::BestMove { mov, ponder } => {
            let mut s = format!("bestmove {}", fmt_move(*mov));
            if let Some(p) = ponder {
                s.push_str(&format!(" ponder {}", fmt_move(*p)));
            }
            s
        }
        UciResponse::CopyProtection(status) => {
            format!("copyprotection {}", fmt_check_status(status))
        }
        UciResponse::Registration(status) => {
            format!("registration {}", fmt_check_status(status))
        }
        UciResponse::Info(f) => fmt_info(f),
        UciResponse::Option { name, option_type } => fmt_option(name, option_type),
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
    moves
        .iter()
        .map(|&m| fmt_move(m))
        .collect::<Vec<_>>()
        .join(" ")
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

fn fmt_info(f: &InfoFields) -> String {
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
    if let Some(pv) = &f.pv {
        out.push_str(&format!(" pv {}", fmt_moves(pv)));
    }
    field!(out, "multipv", f.multipv);
    if let Some(score) = &f.score {
        match score {
            Score::Centipawns { value, bound } => {
                out.push_str(&format!(" score cp {value}{}", fmt_bound(bound)));
            }
            Score::Mate { moves, bound } => {
                out.push_str(&format!(" score mate {moves}{}", fmt_bound(bound)));
            }
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
    if let Some(s) = &f.string {
        out.push_str(&format!(" string {s}"));
    }
    if let Some(r) = &f.refutation {
        out.push_str(&format!(" refutation {}", fmt_move(r.mov)));
        if !r.line.is_empty() {
            out.push_str(&format!(" {}", fmt_moves(&r.line)));
        }
    }
    if let Some(cl) = &f.currline {
        out.push_str(" currline");
        if let Some(cpu) = cl.cpu {
            out.push_str(&format!(" {cpu}"));
        }
        if !cl.moves.is_empty() {
            out.push_str(&format!(" {}", fmt_moves(&cl.moves)));
        }
    }
    out
}

fn fmt_option(name: &str, opt: &OptionType) -> String {
    match opt {
        OptionType::Check { default } => {
            format!("option name {name} type check default {default}")
        }
        OptionType::Spin { default, min, max } => {
            format!("option name {name} type spin default {default} min {min} max {max}")
        }
        OptionType::Combo { default, vars } => {
            let vars: String = vars.iter().map(|v| format!(" var {v}")).collect();
            format!("option name {name} type combo default {default}{vars}")
        }
        OptionType::Button => format!("option name {name} type button"),
        OptionType::Str { default } => {
            let d = default.as_deref().unwrap_or("<empty>");
            format!("option name {name} type string default {d}")
        }
    }
}

pub fn format_request(req: &UciRequest) -> String {
    match req {
        UciRequest::Uci => "uci".to_string(),
        UciRequest::Debug(true) => "debug on".to_string(),
        UciRequest::Debug(false) => "debug off".to_string(),
        UciRequest::IsReady => "isready".to_string(),
        UciRequest::SetOption { name, value } => match value {
            None => format!("setoption name {name}"),
            Some(v) => format!("setoption name {name} value {v}"),
        },
        UciRequest::Register(RegisterCommand::Later) => "register later".to_string(),
        UciRequest::Register(RegisterCommand::Credentials { name, code }) => {
            format!("register name {name} code {code}")
        }
        UciRequest::UciNewGame => "ucinewgame".to_string(),
        UciRequest::Position { start, moves } => {
            let mut s = match start {
                PositionSpec::StartPos => "position startpos".to_string(),
                PositionSpec::Fen(fen) => format!("position fen {fen}"),
            };
            if !moves.is_empty() {
                s.push_str(&format!(" moves {}", fmt_moves(moves)));
            }
            s
        }
        UciRequest::Go(p) => fmt_go(p),
        UciRequest::Stop => "stop".to_string(),
        UciRequest::PonderHit => "ponderhit".to_string(),
        UciRequest::Quit => "quit".to_string(),
    }
}

fn fmt_go(p: &GoParams) -> String {
    let mut s = "go".to_string();
    if p.ponder {
        s.push_str(" ponder");
    }
    match &p.limit {
        SearchLimit::Infinite => s.push_str(" infinite"),
        SearchLimit::Depth(d) => s.push_str(&format!(" depth {d}")),
        SearchLimit::Nodes(n) => s.push_str(&format!(" nodes {n}")),
        SearchLimit::Mate(m) => s.push_str(&format!(" mate {m}")),
        SearchLimit::MoveTime(mt) => s.push_str(&format!(" movetime {mt}")),
        SearchLimit::TimeControl(tc) => {
            if let Some(v) = tc.wtime {
                s.push_str(&format!(" wtime {v}"));
            }
            if let Some(v) = tc.btime {
                s.push_str(&format!(" btime {v}"));
            }
            if let Some(v) = tc.winc {
                s.push_str(&format!(" winc {v}"));
            }
            if let Some(v) = tc.binc {
                s.push_str(&format!(" binc {v}"));
            }
            if let Some(v) = tc.movestogo {
                s.push_str(&format!(" movestogo {v}"));
            }
        }
    }
    if !p.searchmoves.is_empty() {
        s.push_str(&format!(" searchmoves {}", fmt_moves(&p.searchmoves)));
    }
    s
}

// Parsing

pub fn parse_request(line: &str) -> Option<UciRequest> {
    command(line.trim()).map(|(_, req)| req).ok()
}

fn command(i: &str) -> IResult<&str, UciRequest> {
    alt((
        value(UciRequest::UciNewGame, tag("ucinewgame")),
        value(UciRequest::Uci, tag("uci")),
        value(UciRequest::IsReady, tag("isready")),
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
    Ok((
        i,
        UciMove {
            from,
            to,
            promotion,
        },
    ))
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
    Ok((
        i,
        UciRequest::SetOption {
            name: name.trim_end().to_string(),
            value: val.flatten().map(str::to_string),
        },
    ))
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
    Ok((
        i,
        UciRequest::Register(RegisterCommand::Credentials {
            name: name.to_string(),
            code: code.to_string(),
        }),
    ))
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
    Ok((
        i,
        UciRequest::Position {
            start,
            moves: moves.unwrap_or_default(),
        },
    ))
}

fn parse_go(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("go")(i)?;

    let mut searchmoves: Vec<UciMove> = vec![];
    let mut ponder = false;
    let mut wtime: Option<u64> = None;
    let mut btime: Option<u64> = None;
    let mut winc: Option<u64> = None;
    let mut binc: Option<u64> = None;
    let mut movestogo: Option<u64> = None;
    let mut depth: Option<u64> = None;
    let mut nodes: Option<u64> = None;
    let mut mate: Option<u64> = None;
    let mut movetime: Option<u64> = None;
    let mut infinite = false;

    let mut i = i;
    loop {
        let Ok((next, _)) = preceded(
            space1,
            alt((
                map(tag("infinite"), |_| {
                    infinite = true;
                }),
                map(tag("ponder"), |_| {
                    ponder = true;
                }),
                map(preceded(pair(tag("wtime"), space1), parse_u64), |v| {
                    wtime = Some(v);
                }),
                map(preceded(pair(tag("btime"), space1), parse_u64), |v| {
                    btime = Some(v);
                }),
                map(preceded(pair(tag("winc"), space1), parse_u64), |v| {
                    winc = Some(v);
                }),
                map(preceded(pair(tag("binc"), space1), parse_u64), |v| {
                    binc = Some(v);
                }),
                map(preceded(pair(tag("movestogo"), space1), parse_u64), |v| {
                    movestogo = Some(v);
                }),
                map(preceded(pair(tag("depth"), space1), parse_u64), |v| {
                    depth = Some(v);
                }),
                map(preceded(pair(tag("nodes"), space1), parse_u64), |v| {
                    nodes = Some(v);
                }),
                map(preceded(pair(tag("mate"), space1), parse_u64), |v| {
                    mate = Some(v);
                }),
                map(preceded(pair(tag("movetime"), space1), parse_u64), |v| {
                    movetime = Some(v);
                }),
                map(
                    preceded(
                        pair(tag("searchmoves"), space1),
                        separated_list1(space1, parse_move),
                    ),
                    |v| {
                        searchmoves = v;
                    },
                ),
            )),
        )
        .parse(i) else {
            break;
        };
        i = next;
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
        SearchLimit::TimeControl(TimeControl {
            wtime,
            btime,
            winc,
            binc,
            movestogo,
        })
    };

    Ok((
        i,
        UciRequest::Go(GoParams {
            searchmoves,
            ponder,
            limit,
        }),
    ))
}

pub fn parse_response(line: &str) -> Option<UciResponse> {
    response(line.trim()).map(|(_, resp)| resp).ok()
}

fn response(i: &str) -> IResult<&str, UciResponse> {
    alt((
        value(UciResponse::UciOk, tag("uciok")),
        value(UciResponse::ReadyOk, tag("readyok")),
        parse_id,
        parse_bestmove,
        parse_copyprotection,
        parse_registration_resp,
        parse_info_response,
        parse_option_response,
    ))
    .parse(i)
}

fn parse_check_status(i: &str) -> IResult<&str, CheckStatus> {
    alt((
        value(CheckStatus::Checking, tag("checking")),
        value(CheckStatus::Ok, tag("ok")),
        value(CheckStatus::Error, tag("error")),
    ))
    .parse(i)
}

fn parse_id(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("id")(i)?;
    let (i, _) = space1(i)?;
    alt((
        map(preceded(pair(tag("name"), space1), rest), |s: &str| {
            UciResponse::IdName(s.to_string())
        }),
        map(preceded(pair(tag("author"), space1), rest), |s: &str| {
            UciResponse::IdAuthor(s.to_string())
        }),
    ))
    .parse(i)
}

fn parse_bestmove(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("bestmove")(i)?;
    let (i, _) = space1(i)?;
    let (i, mov) = parse_move(i)?;
    let (i, ponder) = opt(preceded(
        pair(space1, tag("ponder")),
        preceded(space1, parse_move),
    ))
    .parse(i)?;
    Ok((i, UciResponse::BestMove { mov, ponder }))
}

fn parse_copyprotection(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("copyprotection")(i)?;
    let (i, _) = space1(i)?;
    map(parse_check_status, UciResponse::CopyProtection).parse(i)
}

fn parse_registration_resp(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("registration")(i)?;
    let (i, _) = space1(i)?;
    map(parse_check_status, UciResponse::Registration).parse(i)
}

fn parse_score_bound(i: &str) -> IResult<&str, ScoreBound> {
    let (i, bound) = opt(preceded(
        space1,
        alt((
            value(ScoreBound::LowerBound, tag("lowerbound")),
            value(ScoreBound::UpperBound, tag("upperbound")),
        )),
    ))
    .parse(i)?;
    Ok((i, bound.unwrap_or(ScoreBound::Exact)))
}

fn parse_score(i: &str) -> IResult<&str, Score> {
    let (i, _) = tag("score")(i)?;
    let (i, _) = space1(i)?;
    alt((
        map(
            pair(
                preceded(pair(tag("cp"), space1), parse_i64),
                parse_score_bound,
            ),
            |(value, bound)| Score::Centipawns {
                value: value as i32,
                bound,
            },
        ),
        map(
            pair(
                preceded(pair(tag("mate"), space1), parse_i64),
                parse_score_bound,
            ),
            |(moves, bound)| Score::Mate {
                moves: moves as i32,
                bound,
            },
        ),
    ))
    .parse(i)
}

fn parse_refutation(i: &str) -> IResult<&str, Refutation> {
    let (i, _) = tag("refutation")(i)?;
    let (i, _) = space1(i)?;
    let (i, mov) = parse_move(i)?;
    let (i, line) = many0(preceded(space1, parse_move)).parse(i)?;
    Ok((i, Refutation { mov, line }))
}

fn parse_currline(i: &str) -> IResult<&str, CurrLine> {
    let (i, _) = tag("currline")(i)?;
    let (i, cpu) = opt(preceded(space1, parse_u64)).parse(i)?;
    let (i, moves) = many0(preceded(space1, parse_move)).parse(i)?;
    Ok((i, CurrLine { cpu, moves }))
}

fn parse_info_response(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("info")(i)?;
    let mut fields = InfoFields::default();

    let mut i = i;
    loop {
        let Ok((next, _)) = preceded(
            space1,
            alt((
                map(preceded(pair(tag("depth"), space1), parse_u64), |v| {
                    fields.depth = Some(v);
                }),
                map(preceded(pair(tag("seldepth"), space1), parse_u64), |v| {
                    fields.seldepth = Some(v);
                }),
                map(preceded(pair(tag("time"), space1), parse_u64), |v| {
                    fields.time = Some(v);
                }),
                map(preceded(pair(tag("nodes"), space1), parse_u64), |v| {
                    fields.nodes = Some(v);
                }),
                map(
                    preceded(pair(tag("pv"), space1), separated_list1(space1, parse_move)),
                    |v| {
                        fields.pv = Some(v);
                    },
                ),
                map(preceded(pair(tag("multipv"), space1), parse_u64), |v| {
                    fields.multipv = Some(v);
                }),
                map(parse_score, |v| {
                    fields.score = Some(v);
                }),
                map(preceded(pair(tag("currmove"), space1), parse_move), |v| {
                    fields.currmove = Some(v);
                }),
                map(
                    preceded(pair(tag("currmovenumber"), space1), parse_u64),
                    |v| {
                        fields.currmovenumber = Some(v);
                    },
                ),
                map(preceded(pair(tag("hashfull"), space1), parse_u64), |v| {
                    fields.hashfull = Some(v);
                }),
                map(preceded(pair(tag("nps"), space1), parse_u64), |v| {
                    fields.nps = Some(v);
                }),
                map(preceded(pair(tag("tbhits"), space1), parse_u64), |v| {
                    fields.tbhits = Some(v);
                }),
                map(preceded(pair(tag("sbhits"), space1), parse_u64), |v| {
                    fields.sbhits = Some(v);
                }),
                map(preceded(pair(tag("cpuload"), space1), parse_u64), |v| {
                    fields.cpuload = Some(v);
                }),
                map(preceded(pair(tag("string"), space1), rest), |v: &str| {
                    fields.string = Some(v.to_string());
                }),
                map(parse_refutation, |v| {
                    fields.refutation = Some(v);
                }),
                map(parse_currline, |v| {
                    fields.currline = Some(v);
                }),
            )),
        )
        .parse(i) else {
            break;
        };
        i = next;
    }

    Ok((i, UciResponse::Info(fields)))
}

fn parse_option_type(i: &str) -> IResult<&str, OptionType> {
    alt((
        parse_check_opt,
        parse_spin_opt,
        parse_combo_opt,
        value(OptionType::Button, tag("button")),
        parse_str_opt,
    ))
    .parse(i)
}

fn parse_check_opt(i: &str) -> IResult<&str, OptionType> {
    let (i, _) = tag("check")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("default")(i)?;
    let (i, _) = space1(i)?;
    let (i, default) = alt((value(true, tag("true")), value(false, tag("false")))).parse(i)?;
    Ok((i, OptionType::Check { default }))
}

fn parse_spin_opt(i: &str) -> IResult<&str, OptionType> {
    let (i, _) = tag("spin")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("default")(i)?;
    let (i, _) = space1(i)?;
    let (i, default) = parse_i64(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("min")(i)?;
    let (i, _) = space1(i)?;
    let (i, min) = parse_i64(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("max")(i)?;
    let (i, _) = space1(i)?;
    let (i, max) = parse_i64(i)?;
    Ok((i, OptionType::Spin { default, min, max }))
}

fn parse_combo_opt(i: &str) -> IResult<&str, OptionType> {
    let (i, _) = tag("combo")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("default")(i)?;
    let (i, _) = space1(i)?;
    let (i, default) = alt((take_until(" var"), rest)).parse(i)?;
    let (i, vars) = many0(preceded(
        pair(space1, tag("var")),
        preceded(space1, alt((take_until(" var"), rest))),
    ))
    .parse(i)?;
    Ok((
        i,
        OptionType::Combo {
            default: default.trim_end().to_string(),
            vars: vars
                .into_iter()
                .map(|s: &str| s.trim_end().to_string())
                .collect(),
        },
    ))
}

fn parse_str_opt(i: &str) -> IResult<&str, OptionType> {
    let (i, _) = tag("string")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("default")(i)?;
    let (i, default) = opt(preceded(space1, rest)).parse(i)?;
    let default = default
        .filter(|s| !s.is_empty() && *s != "<empty>")
        .map(str::to_string);
    Ok((i, OptionType::Str { default }))
}

fn parse_option_response(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("option")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("name")(i)?;
    let (i, _) = space1(i)?;
    let (i, name) = take_until(" type")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("type")(i)?;
    let (i, _) = space1(i)?;
    let (i, option_type) = parse_option_type(i)?;
    Ok((
        i,
        UciResponse::Option {
            name: name.trim_end().to_string(),
            option_type,
        },
    ))
}
