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

pub fn deserialize_request(i: &str) -> IResult<&str, UciRequest> {
    alt((
        value(UciRequest::UciNewGame, tag("ucinewgame")),
        value(UciRequest::Uci, tag("uci")),
        value(UciRequest::IsReady, tag("isready")),
        value(UciRequest::Stop, tag("stop")),
        value(UciRequest::PonderHit, tag("ponderhit")),
        value(UciRequest::Quit, tag("quit")),
        deserialize_debug,
        deserialize_setoption,
        deserialize_register,
        deserialize_position,
        deserialize_go,
    ))
    .parse(i)
}

fn deserialize_square(i: &str) -> IResult<&str, Square> {
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

fn deserialize_move(i: &str) -> IResult<&str, Move> {
    let (i, from) = deserialize_square(i)?;
    let (i, to) = deserialize_square(i)?;
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
        Move {
            from,
            to,
            promotion,
        },
    ))
}

fn deserialize_debug(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("debug")(i)?;
    let (i, _) = space1(i)?;
    alt((
        value(UciRequest::Debug(true), tag("on")),
        value(UciRequest::Debug(false), tag("off")),
    ))
    .parse(i)
}

fn deserialize_setoption(i: &str) -> IResult<&str, UciRequest> {
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

fn deserialize_register(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("register")(i)?;
    let (i, _) = space1(i)?;
    alt((
        value(UciRequest::Register(RegisterCommand::Later), tag("later")),
        deserialize_register_credentials,
    ))
    .parse(i)
}

fn deserialize_register_credentials(i: &str) -> IResult<&str, UciRequest> {
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

fn deserialize_fen_spec(i: &str) -> IResult<&str, PositionSpec> {
    let (i, _) = tag("fen")(i)?;
    let (i, _) = space1(i)?;
    let (i, fen_str) = alt((take_until(" moves"), rest)).parse(i)?;
    let fen = Fen::try_from(fen_str.trim_end())
        .map_err(|_| nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Fail)))?;
    Ok((i, PositionSpec::Fen(fen)))
}

fn deserialize_position(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("position")(i)?;
    let (i, _) = space1(i)?;
    let (i, start) = alt((
        value(PositionSpec::StartPos, tag("startpos")),
        deserialize_fen_spec,
    ))
    .parse(i)?;
    let (i, moves) = opt(preceded(
        pair(space1, tag("moves")),
        many0(preceded(space1, deserialize_move)),
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

fn deserialize_go(i: &str) -> IResult<&str, UciRequest> {
    let (i, _) = tag("go")(i)?;

    let mut searchmoves: Vec<Move> = vec![];
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
                        separated_list1(space1, deserialize_move),
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

pub fn deserialize_response(i: &str) -> IResult<&str, UciResponse> {
    alt((
        value(UciResponse::UciOk, tag("uciok")),
        value(UciResponse::ReadyOk, tag("readyok")),
        deserialize_id,
        deserialize_bestmove,
        deserialize_copyprotection,
        deserialize_registration,
        deserialize_info_response,
        deserialize_option_response,
    ))
    .parse(i)
}

fn deserialize_check_status(i: &str) -> IResult<&str, CheckStatus> {
    alt((
        value(CheckStatus::Checking, tag("checking")),
        value(CheckStatus::Ok, tag("ok")),
        value(CheckStatus::Error, tag("error")),
    ))
    .parse(i)
}

fn deserialize_id(i: &str) -> IResult<&str, UciResponse> {
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

fn deserialize_bestmove(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("bestmove")(i)?;
    let (i, _) = space1(i)?;
    let (i, mov) = deserialize_move(i)?;
    let (i, ponder) = opt(preceded(
        pair(space1, tag("ponder")),
        preceded(space1, deserialize_move),
    ))
    .parse(i)?;
    Ok((i, UciResponse::BestMove { mov, ponder }))
}

fn deserialize_copyprotection(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("copyprotection")(i)?;
    let (i, _) = space1(i)?;
    map(deserialize_check_status, UciResponse::CopyProtection).parse(i)
}

fn deserialize_registration(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("registration")(i)?;
    let (i, _) = space1(i)?;
    map(deserialize_check_status, UciResponse::Registration).parse(i)
}

fn deserialize_score_bound(i: &str) -> IResult<&str, ScoreBound> {
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

fn deserialize_score(i: &str) -> IResult<&str, Score> {
    let (i, _) = tag("score")(i)?;
    let (i, _) = space1(i)?;
    alt((
        map(
            pair(
                preceded(pair(tag("cp"), space1), parse_i64),
                deserialize_score_bound,
            ),
            |(value, bound)| Score::Centipawns {
                value: value as i32,
                bound,
            },
        ),
        map(
            pair(
                preceded(pair(tag("mate"), space1), parse_i64),
                deserialize_score_bound,
            ),
            |(moves, bound)| Score::Mate {
                moves: moves as i32,
                bound,
            },
        ),
    ))
    .parse(i)
}

fn deserialize_refutation(i: &str) -> IResult<&str, Refutation> {
    let (i, _) = tag("refutation")(i)?;
    let (i, _) = space1(i)?;
    let (i, mov) = deserialize_move(i)?;
    let (i, line) = many0(preceded(space1, deserialize_move)).parse(i)?;
    Ok((i, Refutation { mov, line }))
}

fn deserialize_currline(i: &str) -> IResult<&str, CurrLine> {
    let (i, _) = tag("currline")(i)?;
    let (i, cpu) = opt(preceded(space1, parse_u64)).parse(i)?;
    let (i, moves) = many0(preceded(space1, deserialize_move)).parse(i)?;
    Ok((i, CurrLine { cpu, moves }))
}

fn deserialize_info_response(i: &str) -> IResult<&str, UciResponse> {
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
                    preceded(
                        pair(tag("pv"), space1),
                        separated_list1(space1, deserialize_move),
                    ),
                    |v| {
                        fields.pv = Some(v);
                    },
                ),
                map(preceded(pair(tag("multipv"), space1), parse_u64), |v| {
                    fields.multipv = Some(v);
                }),
                map(deserialize_score, |v| {
                    fields.score = Some(v);
                }),
                map(
                    preceded(pair(tag("currmove"), space1), deserialize_move),
                    |v| {
                        fields.currmove = Some(v);
                    },
                ),
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
                map(deserialize_refutation, |v| {
                    fields.refutation = Some(v);
                }),
                map(deserialize_currline, |v| {
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

fn deserialize_option_type(i: &str) -> IResult<&str, OptionType> {
    alt((
        deserialize_check_opt,
        deserialize_spin_opt,
        deserialize_combo_opt,
        value(OptionType::Button, tag("button")),
        deserialize_str_opt,
    ))
    .parse(i)
}

fn deserialize_check_opt(i: &str) -> IResult<&str, OptionType> {
    let (i, _) = tag("check")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("default")(i)?;
    let (i, _) = space1(i)?;
    let (i, default) = alt((value(true, tag("true")), value(false, tag("false")))).parse(i)?;
    Ok((i, OptionType::Check { default }))
}

fn deserialize_spin_opt(i: &str) -> IResult<&str, OptionType> {
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

fn deserialize_combo_opt(i: &str) -> IResult<&str, OptionType> {
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

fn deserialize_str_opt(i: &str) -> IResult<&str, OptionType> {
    let (i, _) = tag("string")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("default")(i)?;
    let (i, default) = opt(preceded(space1, rest)).parse(i)?;
    let default = default
        .filter(|s| !s.is_empty() && *s != "<empty>")
        .map(str::to_string);
    Ok((i, OptionType::Str { default }))
}

fn deserialize_option_response(i: &str) -> IResult<&str, UciResponse> {
    let (i, _) = tag("option")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("name")(i)?;
    let (i, _) = space1(i)?;
    let (i, name) = take_until(" type")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("type")(i)?;
    let (i, _) = space1(i)?;
    let (i, option_type) = deserialize_option_type(i)?;
    Ok((
        i,
        UciResponse::Option {
            name: name.trim_end().to_string(),
            option_type,
        },
    ))
}
