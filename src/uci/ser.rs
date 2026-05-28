use super::*;

pub fn serialize_request(req: &UciRequest) -> String {
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
                s.push_str(&format!(" moves {}", serialize_moves(moves)));
            }
            s
        }
        UciRequest::Go(p) => serialize_go(p),
        UciRequest::Stop => "stop".to_string(),
        UciRequest::PonderHit => "ponderhit".to_string(),
        UciRequest::Quit => "quit".to_string(),
    }
}

fn serialize_go(p: &GoParams) -> String {
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
        s.push_str(&format!(" searchmoves {}", serialize_moves(&p.searchmoves)));
    }
    s
}

pub fn serialize_response(resp: &UciResponse) -> String {
    match resp {
        UciResponse::IdName(name) => format!("id name {name}"),
        UciResponse::IdAuthor(author) => format!("id author {author}"),
        UciResponse::UciOk => "uciok".to_string(),
        UciResponse::ReadyOk => "readyok".to_string(),
        UciResponse::BestMove { mov, ponder } => {
            let mut s = format!("bestmove {}", serialize_move(*mov));
            if let Some(p) = ponder {
                s.push_str(&format!(" ponder {}", serialize_move(*p)));
            }
            s
        }
        UciResponse::CopyProtection(status) => {
            format!("copyprotection {}", serialize_check_status(status))
        }
        UciResponse::Registration(status) => {
            format!("registration {}", serialize_check_status(status))
        }
        UciResponse::Info(f) => serialize_info(f),
        UciResponse::Option { name, option_type } => serialize_option(name, option_type),
    }
}

fn serialize_move(m: Move) -> String {
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

fn serialize_moves(moves: &[Move]) -> String {
    moves
        .iter()
        .map(|&m| serialize_move(m))
        .collect::<Vec<_>>()
        .join(" ")
}

fn serialize_check_status(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Checking => "checking",
        CheckStatus::Ok => "ok",
        CheckStatus::Error => "error",
    }
}

fn serialize_score_bound(b: &ScoreBound) -> &'static str {
    match b {
        ScoreBound::Exact => "",
        ScoreBound::LowerBound => " lowerbound",
        ScoreBound::UpperBound => " upperbound",
    }
}

fn serialize_info(f: &InfoFields) -> String {
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
        out.push_str(&format!(" pv {}", serialize_moves(pv)));
    }
    field!(out, "multipv", f.multipv);
    if let Some(score) = &f.score {
        match score {
            Score::Centipawns { value, bound } => {
                out.push_str(&format!(
                    " score cp {value}{}",
                    serialize_score_bound(bound)
                ));
            }
            Score::Mate { moves, bound } => {
                out.push_str(&format!(
                    " score mate {moves}{}",
                    serialize_score_bound(bound)
                ));
            }
        }
    }
    if let Some(m) = f.currmove {
        out.push_str(&format!(" currmove {}", serialize_move(m)));
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
        out.push_str(&format!(" refutation {}", serialize_move(r.mov)));
        if !r.line.is_empty() {
            out.push_str(&format!(" {}", serialize_moves(&r.line)));
        }
    }
    if let Some(cl) = &f.currline {
        out.push_str(" currline");
        if let Some(cpu) = cl.cpu {
            out.push_str(&format!(" {cpu}"));
        }
        if !cl.moves.is_empty() {
            out.push_str(&format!(" {}", serialize_moves(&cl.moves)));
        }
    }
    out
}

fn serialize_option(name: &str, opt: &OptionType) -> String {
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
