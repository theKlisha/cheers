use super::*;

fn sq(file: File, rank: Rank) -> Square {
    Square { file, rank }
}

fn mv(from: Square, to: Square) -> UciMove {
    UciMove {
        from,
        to,
        promotion: None,
    }
}

fn mvp(from: Square, to: Square, promo: Promotion) -> UciMove {
    UciMove {
        from,
        to,
        promotion: Some(promo),
    }
}

fn roundtrip_request(req: UciRequest) {
    let text = String::from(&req);
    let parsed = UciRequest::try_from(text.as_str())
        .unwrap_or_else(|_| panic!("roundtrip failed for: {text:?}"));
    assert_eq!(req, parsed);
}

fn roundtrip_response(resp: UciResponse) {
    let text = String::from(&resp);
    let parsed = UciResponse::try_from(text.as_str())
        .unwrap_or_else(|_| panic!("roundtrip failed for: {text:?}"));
    assert_eq!(resp, parsed);
}

// UciRequest roundtrips

#[test]
fn request_uci() {
    roundtrip_request(UciRequest::Uci);
}

#[test]
fn request_debug_on() {
    roundtrip_request(UciRequest::Debug(true));
}

#[test]
fn request_debug_off() {
    roundtrip_request(UciRequest::Debug(false));
}

#[test]
fn request_isready() {
    roundtrip_request(UciRequest::IsReady);
}

#[test]
fn request_setoption_no_value() {
    roundtrip_request(UciRequest::SetOption {
        name: "Hash".to_string(),
        value: None,
    });
}

#[test]
fn request_setoption_with_value() {
    roundtrip_request(UciRequest::SetOption {
        name: "Hash".to_string(),
        value: Some("128".to_string()),
    });
}

#[test]
fn request_setoption_multiword_name() {
    roundtrip_request(UciRequest::SetOption {
        name: "Clear Hash".to_string(),
        value: None,
    });
}

#[test]
fn request_register_later() {
    roundtrip_request(UciRequest::Register(RegisterCommand::Later));
}

#[test]
fn request_register_credentials() {
    roundtrip_request(UciRequest::Register(RegisterCommand::Credentials {
        name: "Stefan MK".to_string(),
        code: "4359874324".to_string(),
    }));
}

#[test]
fn request_ucinewgame() {
    roundtrip_request(UciRequest::UciNewGame);
}

#[test]
fn request_position_startpos_no_moves() {
    roundtrip_request(UciRequest::Position {
        start: PositionSpec::StartPos,
        moves: vec![],
    });
}

#[test]
fn request_position_startpos_with_moves() {
    roundtrip_request(UciRequest::Position {
        start: PositionSpec::StartPos,
        moves: vec![
            mv(sq(File::E, Rank::R2), sq(File::E, Rank::R4)),
            mv(sq(File::E, Rank::R7), sq(File::E, Rank::R5)),
        ],
    });
}

#[test]
fn request_position_fen_no_moves() {
    roundtrip_request(UciRequest::Position {
        start: PositionSpec::Fen(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string(),
        ),
        moves: vec![],
    });
}

#[test]
fn request_position_fen_with_moves() {
    roundtrip_request(UciRequest::Position {
        start: PositionSpec::Fen("8/8/8/8/8/8/8/8 w - - 0 1".to_string()),
        moves: vec![mvp(
            sq(File::A, Rank::R7),
            sq(File::A, Rank::R8),
            Promotion::Queen,
        )],
    });
}

#[test]
fn request_go_infinite() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![],
        ponder: false,
        limit: SearchLimit::Infinite,
    }));
}

#[test]
fn request_go_depth() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![],
        ponder: false,
        limit: SearchLimit::Depth(10),
    }));
}

#[test]
fn request_go_nodes() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![],
        ponder: false,
        limit: SearchLimit::Nodes(1_000_000),
    }));
}

#[test]
fn request_go_mate() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![],
        ponder: false,
        limit: SearchLimit::Mate(5),
    }));
}

#[test]
fn request_go_movetime() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![],
        ponder: false,
        limit: SearchLimit::MoveTime(3000),
    }));
}

#[test]
fn request_go_time_control() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![],
        ponder: false,
        limit: SearchLimit::TimeControl(TimeControl {
            wtime: Some(300000),
            btime: Some(300000),
            winc: Some(1000),
            binc: Some(1000),
            movestogo: Some(40),
        }),
    }));
}

#[test]
fn request_go_ponder() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![],
        ponder: true,
        limit: SearchLimit::Infinite,
    }));
}

#[test]
fn request_go_searchmoves() {
    roundtrip_request(UciRequest::Go(GoParams {
        searchmoves: vec![
            mv(sq(File::E, Rank::R2), sq(File::E, Rank::R4)),
            mv(sq(File::D, Rank::R2), sq(File::D, Rank::R4)),
        ],
        ponder: false,
        limit: SearchLimit::Depth(6),
    }));
}

#[test]
fn request_stop() {
    roundtrip_request(UciRequest::Stop);
}

#[test]
fn request_ponderhit() {
    roundtrip_request(UciRequest::PonderHit);
}

#[test]
fn request_quit() {
    roundtrip_request(UciRequest::Quit);
}

// UciResponse roundtrips

#[test]
fn response_id_name() {
    roundtrip_response(UciResponse::IdName("Stockfish 16".to_string()));
}

#[test]
fn response_id_author() {
    roundtrip_response(UciResponse::IdAuthor(
        "T. Romstad, M. Costalba, J. Kiiski, G. Linscott".to_string(),
    ));
}

#[test]
fn response_uciok() {
    roundtrip_response(UciResponse::UciOk);
}

#[test]
fn response_readyok() {
    roundtrip_response(UciResponse::ReadyOk);
}

#[test]
fn response_bestmove_no_ponder() {
    roundtrip_response(UciResponse::BestMove {
        mov: mv(sq(File::E, Rank::R2), sq(File::E, Rank::R4)),
        ponder: None,
    });
}

#[test]
fn response_bestmove_with_ponder() {
    roundtrip_response(UciResponse::BestMove {
        mov: mv(sq(File::E, Rank::R2), sq(File::E, Rank::R4)),
        ponder: Some(mv(sq(File::E, Rank::R7), sq(File::E, Rank::R5))),
    });
}

#[test]
fn response_bestmove_promotion() {
    roundtrip_response(UciResponse::BestMove {
        mov: mvp(
            sq(File::A, Rank::R7),
            sq(File::A, Rank::R8),
            Promotion::Queen,
        ),
        ponder: None,
    });
}

#[test]
fn response_copyprotection_checking() {
    roundtrip_response(UciResponse::CopyProtection(CheckStatus::Checking));
}

#[test]
fn response_copyprotection_ok() {
    roundtrip_response(UciResponse::CopyProtection(CheckStatus::Ok));
}

#[test]
fn response_copyprotection_error() {
    roundtrip_response(UciResponse::CopyProtection(CheckStatus::Error));
}

#[test]
fn response_registration_checking() {
    roundtrip_response(UciResponse::Registration(CheckStatus::Checking));
}

#[test]
fn response_registration_ok() {
    roundtrip_response(UciResponse::Registration(CheckStatus::Ok));
}

#[test]
fn response_registration_error() {
    roundtrip_response(UciResponse::Registration(CheckStatus::Error));
}

#[test]
fn response_info_empty() {
    roundtrip_response(UciResponse::Info(InfoFields::default()));
}

#[test]
fn response_info_depth_nodes() {
    roundtrip_response(UciResponse::Info(InfoFields {
        depth: Some(10),
        seldepth: Some(15),
        nodes: Some(500_000),
        time: Some(1234),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_score_cp_exact() {
    roundtrip_response(UciResponse::Info(InfoFields {
        score: Some(Score::Centipawns {
            value: 42,
            bound: ScoreBound::Exact,
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_score_cp_lowerbound() {
    roundtrip_response(UciResponse::Info(InfoFields {
        score: Some(Score::Centipawns {
            value: -100,
            bound: ScoreBound::LowerBound,
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_score_cp_upperbound() {
    roundtrip_response(UciResponse::Info(InfoFields {
        score: Some(Score::Centipawns {
            value: 300,
            bound: ScoreBound::UpperBound,
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_score_mate() {
    roundtrip_response(UciResponse::Info(InfoFields {
        score: Some(Score::Mate {
            moves: 3,
            bound: ScoreBound::Exact,
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_score_mate_negative() {
    roundtrip_response(UciResponse::Info(InfoFields {
        score: Some(Score::Mate {
            moves: -2,
            bound: ScoreBound::Exact,
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_pv() {
    roundtrip_response(UciResponse::Info(InfoFields {
        pv: Some(vec![
            mv(sq(File::E, Rank::R2), sq(File::E, Rank::R4)),
            mv(sq(File::E, Rank::R7), sq(File::E, Rank::R5)),
            mv(sq(File::G, Rank::R1), sq(File::F, Rank::R3)),
        ]),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_currmove() {
    roundtrip_response(UciResponse::Info(InfoFields {
        currmove: Some(mv(sq(File::D, Rank::R2), sq(File::D, Rank::R4))),
        currmovenumber: Some(3),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_hash_nps() {
    roundtrip_response(UciResponse::Info(InfoFields {
        hashfull: Some(500),
        nps: Some(1_234_567),
        tbhits: Some(42),
        sbhits: Some(0),
        cpuload: Some(750),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_string() {
    roundtrip_response(UciResponse::Info(InfoFields {
        string: Some("current move: e2e4".to_string()),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_refutation_no_line() {
    roundtrip_response(UciResponse::Info(InfoFields {
        refutation: Some(Refutation {
            mov: mv(sq(File::D, Rank::R1), sq(File::H, Rank::R5)),
            line: vec![],
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_refutation_with_line() {
    roundtrip_response(UciResponse::Info(InfoFields {
        refutation: Some(Refutation {
            mov: mv(sq(File::D, Rank::R1), sq(File::H, Rank::R5)),
            line: vec![mv(sq(File::G, Rank::R7), sq(File::G, Rank::R6))],
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_currline_no_cpu() {
    roundtrip_response(UciResponse::Info(InfoFields {
        currline: Some(CurrLine {
            cpu: None,
            moves: vec![mv(sq(File::E, Rank::R2), sq(File::E, Rank::R4))],
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_currline_with_cpu() {
    roundtrip_response(UciResponse::Info(InfoFields {
        currline: Some(CurrLine {
            cpu: Some(1),
            moves: vec![
                mv(sq(File::E, Rank::R2), sq(File::E, Rank::R4)),
                mv(sq(File::E, Rank::R7), sq(File::E, Rank::R5)),
            ],
        }),
        ..InfoFields::default()
    }));
}

#[test]
fn response_info_multipv() {
    roundtrip_response(UciResponse::Info(InfoFields {
        multipv: Some(2),
        ..InfoFields::default()
    }));
}

#[test]
fn response_option_check() {
    roundtrip_response(UciResponse::Option {
        name: "Ponder".to_string(),
        option_type: OptionType::Check { default: false },
    });
}

#[test]
fn response_option_check_true() {
    roundtrip_response(UciResponse::Option {
        name: "Ponder".to_string(),
        option_type: OptionType::Check { default: true },
    });
}

#[test]
fn response_option_spin() {
    roundtrip_response(UciResponse::Option {
        name: "Hash".to_string(),
        option_type: OptionType::Spin {
            default: 16,
            min: 1,
            max: 65536,
        },
    });
}

#[test]
fn response_option_spin_negative() {
    roundtrip_response(UciResponse::Option {
        name: "Contempt".to_string(),
        option_type: OptionType::Spin {
            default: 0,
            min: -100,
            max: 100,
        },
    });
}

#[test]
fn response_option_combo_no_vars() {
    roundtrip_response(UciResponse::Option {
        name: "Style".to_string(),
        option_type: OptionType::Combo {
            default: "Normal".to_string(),
            vars: vec![],
        },
    });
}

#[test]
fn response_option_combo_with_vars() {
    roundtrip_response(UciResponse::Option {
        name: "Style".to_string(),
        option_type: OptionType::Combo {
            default: "Normal".to_string(),
            vars: vec![
                "Solid".to_string(),
                "Normal".to_string(),
                "Risky".to_string(),
            ],
        },
    });
}

#[test]
fn response_option_button() {
    roundtrip_response(UciResponse::Option {
        name: "Clear Hash".to_string(),
        option_type: OptionType::Button,
    });
}

#[test]
fn response_option_string_no_default() {
    roundtrip_response(UciResponse::Option {
        name: "NalimovPath".to_string(),
        option_type: OptionType::Str { default: None },
    });
}

#[test]
fn response_option_string_with_default() {
    roundtrip_response(UciResponse::Option {
        name: "NalimovPath".to_string(),
        option_type: OptionType::Str {
            default: Some("c:\\".to_string()),
        },
    });
}
