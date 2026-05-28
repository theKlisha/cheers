use super::*;
use crate::{CastlingRights, Color, File, Rank, Square};

fn roundtrip(s: &str) {
    let fen = Fen::try_from(s)
        .unwrap_or_else(|e| panic!("parse failed for {s:?}: {e}"));
    assert_eq!(fen.to_string(), s, "roundtrip mismatch for {s:?}");
}

#[test]
fn roundtrip_startpos() {
    roundtrip("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}

#[test]
fn roundtrip_empty_board() {
    roundtrip("8/8/8/8/8/8/8/8 w - - 0 1");
}

#[test]
fn roundtrip_en_passant() {
    roundtrip("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
}

#[test]
fn roundtrip_en_passant_black() {
    roundtrip("rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR w KQkq f3 0 2");
}

#[test]
fn roundtrip_no_castling() {
    roundtrip("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1");
}

#[test]
fn roundtrip_partial_castling() {
    roundtrip("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kq - 4 10");
}

#[test]
fn roundtrip_midgame() {
    roundtrip("r1bqkb1r/pppppppp/2n2n2/8/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3");
}

#[test]
fn roundtrip_kiwipete() {
    roundtrip("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
}

#[test]
fn roundtrip_fools_mate() {
    roundtrip("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
}

#[test]
fn parse_side_white() {
    let fen = Fen::try_from("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
    assert_eq!(fen.side_to_move, Color::White);
}

#[test]
fn parse_side_black() {
    let fen = Fen::try_from("8/8/8/8/8/8/8/8 b - - 0 1").unwrap();
    assert_eq!(fen.side_to_move, Color::Black);
}

#[test]
fn parse_castling_all() {
    let fen = Fen::try_from("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
    assert_eq!(fen.castling, CastlingRights::all());
}

#[test]
fn parse_castling_none() {
    let fen = Fen::try_from("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
    assert_eq!(fen.castling, CastlingRights::none());
}

#[test]
fn parse_castling_partial() {
    let fen = Fen::try_from("8/8/8/8/8/8/8/8 w Kq - 0 1").unwrap();
    assert!(fen.castling.white_kingside);
    assert!(!fen.castling.white_queenside);
    assert!(!fen.castling.black_kingside);
    assert!(fen.castling.black_queenside);
}

#[test]
fn parse_en_passant_present() {
    let fen =
        Fen::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
    assert_eq!(
        fen.en_passant,
        Some(Square { file: File::E, rank: Rank::R3 })
    );
}

#[test]
fn parse_en_passant_absent() {
    let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    assert_eq!(fen.en_passant, None);
}

#[test]
fn parse_clocks() {
    let fen = Fen::try_from("8/8/8/8/8/8/8/8 w - - 7 42").unwrap();
    assert_eq!(fen.halfmove_clock, 7);
    assert_eq!(fen.fullmove_number, 42);
}

#[test]
fn error_too_few_fields() {
    assert!(Fen::try_from("8/8/8/8/8/8/8/8 w").is_err());
}

#[test]
fn error_invalid_piece() {
    assert!(Fen::try_from("8/8/8/8/8/8/8/x7 w - - 0 1").is_err());
}

#[test]
fn error_invalid_side() {
    assert!(Fen::try_from("8/8/8/8/8/8/8/8 x - - 0 1").is_err());
}

#[test]
fn error_invalid_en_passant() {
    assert!(Fen::try_from("8/8/8/8/8/8/8/8 w - z9 0 1").is_err());
}
