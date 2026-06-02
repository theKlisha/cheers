macro_rules! fen {
    ($s:expr) => {
        Fen::try_from($s).unwrap()
    };
}

macro_rules! board_tests {
    ($name:ident, $board:ty) => {
        mod $name {
            use super::*;
            use $crate::{Board, Color, Fen, File, Kind, Move, Promotion, Rank, SquareName as Sq};

            fn kiwipete() -> Fen {
                fen!("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            }

            fn has_move(moves: &[Move], m: Move) -> bool {
                moves.contains(&m)
            }

            fn perft(b: &impl Board, depth: u32) -> u64 {
                if depth == 0 {
                    return 1;
                }
                let moves: Vec<Move> = b.move_iter().collect();
                if depth == 1 {
                    return moves.len() as u64;
                }
                let mut count = 0u64;
                for m in moves {
                    let child = b.do_move(m);
                    count += perft(&child, depth - 1);
                }
                count
            }

            #[test]
            fn startpos_fen_roundtrip() {
                assert_eq!(<$board>::startpos().fen(), Fen::startpos());
            }

            #[test]
            fn fen_roundtrip_midgame() {
                let f = fen!("r1bqkb1r/pppppppp/2n2n2/8/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3");
                assert_eq!(<$board>::from_fen(&f).fen(), f);
            }

            #[test]
            fn fen_roundtrip_with_en_passant() {
                let f = fen!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
                assert_eq!(<$board>::from_fen(&f).fen(), f);
            }

            #[test]
            fn fen_roundtrip_no_castling() {
                let f = fen!("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1");
                assert_eq!(<$board>::from_fen(&f).fen(), f);
            }

            #[test]
            fn piece_at_startpos() {
                let board = <$board>::startpos();
                assert_eq!(board.piece_at(Sq::A1), Some((Color::White, Kind::Rook)));
                assert_eq!(board.piece_at(Sq::A8), Some((Color::Black, Kind::Rook)));
                assert_eq!(board.piece_at(Sq::E1), Some((Color::White, Kind::King)));
                assert_eq!(board.piece_at(Sq::E8), Some((Color::Black, Kind::King)));
                assert_eq!(board.piece_at(Sq::F2), Some((Color::White, Kind::Pawn)));
                assert_eq!(board.piece_at(Sq::F7), Some((Color::Black, Kind::Pawn)));
                assert_eq!(board.piece_at(Sq::D4), None);
            }

            #[test]
            fn make_move_e2e4() {
                let board = <$board>::startpos();
                let board = board.do_move(((File::E, Rank::R2), (File::E, Rank::R4)));

                let f = fen!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_sequence_italian() {
                let board = <$board>::startpos();
                let board = board.do_move(((Sq::E2), (Sq::E4)));
                let board = board.do_move(((Sq::E7), (Sq::E5)));
                let board = board.do_move(((Sq::G1), (Sq::F3)));
                let board = board.do_move(((Sq::B8), (Sq::C6)));

                let f = fen!("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_capture_resets_halfmove() {
                let f = fen!("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::E4, Sq::D5));
                assert_eq!(board.piece_at(Sq::D5), Some((Color::White, Kind::Pawn)));
                assert_eq!(board.piece_at(Sq::E4), None);

                let f = fen!("rnbqkbnr/ppp1pppp/8/3P4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_en_passant_white() {
                let f = fen!("rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::F5, Sq::E6));
                assert_eq!(board.piece_at(Sq::E6), Some((Color::White, Kind::Pawn)));
                assert_eq!(board.piece_at(Sq::E5), None);
                assert_eq!(board.piece_at(Sq::F5), None);
            }

            #[test]
            fn make_move_en_passant_black() {
                let f = fen!("rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::F4, Sq::E3));
                assert_eq!(board.piece_at(Sq::E3), Some((Color::Black, Kind::Pawn)));
                assert_eq!(board.piece_at(Sq::E4), None);
                assert_eq!(board.piece_at(Sq::F4), None);
            }

            #[test]
            fn make_move_kingside_castle_white() {
                let f = fen!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::E1, Sq::G1));
                assert_eq!(board.piece_at(Sq::G1), Some((Color::White, Kind::King)));
                assert_eq!(board.piece_at(Sq::F1), Some((Color::White, Kind::Rook)));
                assert_eq!(board.piece_at(Sq::E1), None);
                assert_eq!(board.piece_at(Sq::H1), None);
                let f = fen!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 b kq - 1 1");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_queenside_castle_white() {
                let f = fen!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3KBNR w KQkq - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::E1, Sq::C1));
                assert_eq!(board.piece_at(Sq::C1), Some((Color::White, Kind::King)));
                assert_eq!(board.piece_at(Sq::D1), Some((Color::White, Kind::Rook)));
                assert_eq!(board.piece_at(Sq::E1), None);
                assert_eq!(board.piece_at(Sq::A1), None);
                let f = fen!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/2KR1BNR b kq - 1 1");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_kingside_castle_black() {
                let f = fen!("rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::E8, Sq::G8));
                assert_eq!(board.piece_at(Sq::G8), Some((Color::Black, Kind::King)));
                assert_eq!(board.piece_at(Sq::F8), Some((Color::Black, Kind::Rook)));
                assert_eq!(board.piece_at(Sq::E8), None);
                assert_eq!(board.piece_at(Sq::H8), None);
                let f = fen!("rnbq1rk1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 1 2");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_promotion() {
                let f = fen!("8/4P3/8/8/8/8/8/4K2k w - - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::E7, Sq::E8, Promotion::Queen));
                assert_eq!(board.piece_at(Sq::E8), Some((Color::White, Kind::Queen)));
                assert_eq!(board.piece_at(Sq::E7), None);
            }

            #[test]
            fn make_move_promotion_capture() {
                let f = fen!("3r4/4P3/8/8/8/8/8/4K2k w - - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::E7, Sq::D8, Promotion::Knight));
                assert_eq!(board.piece_at(Sq::D8), Some((Color::White, Kind::Knight)));
            }

            #[test]
            fn make_move_rook_move_removes_castling() {
                let f = fen!("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::H1, Sq::H2));
                let f = fen!("r3k2r/pppppppp/8/8/8/8/PPPPPPPR/R3K3 b Qkq - 0 1");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_rook_captured_removes_castling() {
                let f = fen!("r3k2r/pppppppp/8/8/8/7B/PPPPPPPP/R3K2R w KQkq - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::H3, Sq::A8));
                let f = fen!("B3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQk - 0 1");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_fullmove_increments_after_black() {
                let board = <$board>::startpos();
                let board = board.do_move((Sq::E2, Sq::E4));
                let f = fen!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
                assert_eq!(board.fen(), f);
                let board = board.do_move((Sq::E7, Sq::E5));
                let f = fen!("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn make_move_halfmove_increments_on_quiet() {
                let f = fen!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
                let board = <$board>::from_fen(&f);
                let board = board.do_move((Sq::B8, Sq::C6));
                let f = fen!("r1bqkbnr/pppppppp/2n5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn double_push_sets_en_passant() {
                let board = <$board>::startpos();
                let board = board.do_move((Sq::D2, Sq::D4));
                let f = fen!("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn en_passant_cleared_after_next_move() {
                let board = <$board>::startpos();
                let board = board.do_move((Sq::E2, Sq::E4));
                let f = fen!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
                assert_eq!(board.fen(), f);
                let board = board.do_move((Sq::B8, Sq::C6));
                let f = fen!("r1bqkbnr/pppppppp/2n5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2");
                assert_eq!(board.fen(), f);
            }

            #[test]
            fn is_in_check_startpos() {
                let board = <$board>::startpos();
                assert_eq!(board.check(), None);
            }

            #[test]
            fn is_in_check_fools_mate() {
                let f = fen!("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
                let board = <$board>::from_fen(&f);
                assert_eq!(board.check(), Some(Color::White));
                assert_ne!(board.check(), Some(Color::Black));
            }

            #[test]
            fn generate_moves_startpos_count() {
                let board = <$board>::startpos();
                assert_eq!(board.move_iter().count(), 20);
            }

            #[test]
            fn generate_moves_includes_castling() {
                let f = fen!("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
                let board = <$board>::from_fen(&f);
                let moves: Vec<Move> = board.move_iter().collect();
                assert!(has_move(&moves, (Sq::E1, Sq::G1).into()));
                assert!(has_move(&moves, (Sq::E1, Sq::C1).into()));
            }

            #[test]
            fn generate_moves_no_castling_through_check() {
                let f = fen!("4k3/8/8/8/5r2/8/8/R3K2R w KQ - 0 1");
                let board = <$board>::from_fen(&f);
                let moves: Vec<Move> = board.move_iter().collect();
                assert!(!has_move(&moves, (Sq::E1, Sq::G1).into()));
                assert!(has_move(&moves, (Sq::E1, Sq::C1).into()));
            }

            #[test]
            fn generate_moves_no_castling_in_check() {
                let f = fen!("4k3/8/8/8/4r3/8/8/R3K2R w KQ - 0 1");
                let board = <$board>::from_fen(&f);
                let moves: Vec<Move> = board.move_iter().collect();
                assert!(!has_move(&moves, (Sq::E1, Sq::G1).into()));
                assert!(!has_move(&moves, (Sq::E1, Sq::C1).into()));
            }

            #[test]
            fn generate_moves_en_passant() {
                let f = fen!("rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3");
                let board = <$board>::from_fen(&f);
                let moves: Vec<Move> = board.move_iter().collect();
                assert!(has_move(&moves, (Sq::F5, Sq::E6).into()));
            }

            #[test]
            fn generate_moves_promotion() {
                let f = fen!("8/4P3/8/8/8/8/8/4K2k w - - 0 1");
                let board = <$board>::from_fen(&f);
                let moves: Vec<Move> = board.move_iter().collect();
                assert!(has_move(&moves, (Sq::E7, Sq::E8, Promotion::Queen).into()));
                assert!(has_move(&moves, (Sq::E7, Sq::E8, Promotion::Rook).into()));
                assert!(has_move(&moves, (Sq::E7, Sq::E8, Promotion::Bishop).into()));
                assert!(has_move(&moves, (Sq::E7, Sq::E8, Promotion::Knight).into()));
            }

            #[test]
            fn generate_moves_pinned_piece() {
                let f = fen!("8/8/8/8/8/1b6/2P5/3K3k w - - 0 1");
                let board = <$board>::from_fen(&f);
                let moves: Vec<Move> = board.move_iter().collect();
                assert!(!has_move(&moves, (Sq::C2, Sq::C3).into()));
                assert!(!has_move(&moves, (Sq::C2, Sq::C4).into()));
            }

            #[test]
            fn generate_moves_checkmate_no_moves() {
                let f = fen!("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
                let board = <$board>::from_fen(&f);
                assert_eq!(board.move_iter().count(), 0);
                assert_eq!(board.mate(), Some(Color::White));
            }

            #[test]
            fn generate_moves_stalemate_no_moves() {
                let f = fen!("7k/8/6QK/8/8/8/8/8 b - - 0 1");
                let board = <$board>::from_fen(&f);
                assert_eq!(board.move_iter().count(), 0);
                assert_eq!(board.mate(), None);
            }

            #[test]
            fn perft_startpos_depth1() {
                assert_eq!(perft(&<$board>::startpos(), 1), 20);
            }

            #[test]
            fn perft_startpos_depth2() {
                assert_eq!(perft(&<$board>::startpos(), 2), 400);
            }

            #[test]
            fn perft_startpos_depth3() {
                assert_eq!(perft(&<$board>::startpos(), 3), 8902);
            }

            #[test]
            fn perft_startpos_depth4() {
                assert_eq!(perft(&<$board>::startpos(), 4), 197281);
            }

            #[test]
            fn perft_kiwipete_depth1() {
                assert_eq!(perft(&<$board>::from_fen(&kiwipete()), 1), 48);
            }

            #[test]
            fn perft_kiwipete_depth2() {
                assert_eq!(perft(&<$board>::from_fen(&kiwipete()), 2), 2039);
            }

            #[test]
            fn perft_kiwipete_depth3() {
                assert_eq!(perft(&<$board>::from_fen(&kiwipete()), 3), 97862);
            }
        }
    };
}

use super::mailbox::Mailbox;
board_tests!(mailbox, Mailbox);
