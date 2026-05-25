use crate::board::{Board, Color, Kind, Move, Piece};

pub fn sq(file: u8, rank: u8) -> u8 {
    rank * 8 + file
}

pub fn file_of(sq: u8) -> u8 {
    sq % 8
}

pub fn rank_of(sq: u8) -> u8 {
    sq / 8
}

pub fn parse_square(s: &str) -> Result<u8, String> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return Err(format!("invalid square: {}", s));
    }
    let file = bytes[0].wrapping_sub(b'a');
    let rank = bytes[1].wrapping_sub(b'1');
    if file >= 8 || rank >= 8 {
        return Err(format!("invalid square: {}", s));
    }
    Ok(sq(file, rank))
}

fn char_to_piece(ch: char) -> Result<Piece, String> {
    let color = if ch.is_uppercase() { Color::White } else { Color::Black };
    let kind = match ch.to_ascii_lowercase() {
        'p' => Kind::Pawn,
        'r' => Kind::Rook,
        'n' => Kind::Kingt,
        'b' => Kind::Bishop,
        'q' => Kind::Queen,
        'k' => Kind::King,
        _ => return Err(format!("invalid piece char: {}", ch)),
    };
    Ok((color, kind))
}

fn piece_to_char(piece: Piece) -> char {
    let ch = match piece.1 {
        Kind::Pawn => 'p',
        Kind::Rook => 'r',
        Kind::Kingt => 'n',
        Kind::Bishop => 'b',
        Kind::Queen => 'q',
        Kind::King => 'k',
    };
    match piece.0 {
        Color::White => ch.to_ascii_uppercase(),
        Color::Black => ch,
    }
}

const DIAGONALS: [(i8, i8); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
const ORTHOGONALS: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const KNIGHT_JUMPS: [(i8, i8); 8] = [
    (-2, -1), (-2, 1), (-1, -2), (-1, 2),
    (1, -2), (1, 2), (2, -1), (2, 1),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn none() -> Self {
        CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    pub fn all() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
    pub squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Mailbox {
    pub fn empty() -> Self {
        Mailbox {
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::none(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn set_piece(&mut self, sq: u8, piece: Option<Piece>) {
        self.squares[sq as usize] = piece;
    }

    pub fn find_king(&self, color: Color) -> Option<u8> {
        for i in 0..64u8 {
            if let Some((c, Kind::King)) = self.piece_at(i) {
                if c == color {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn is_attacked(&self, target: u8, by_color: Color) -> bool {
        let tf = file_of(target) as i8;
        let tr = rank_of(target) as i8;

        let pawn_rank_offset: i8 = match by_color {
            Color::White => -1,
            Color::Black => 1,
        };
        for df in [-1i8, 1] {
            let f = tf + df;
            let r = tr + pawn_rank_offset;
            if f >= 0 && f < 8 && r >= 0 && r < 8 {
                if self.piece_at(sq(f as u8, r as u8)) == Some((by_color, Kind::Pawn)) {
                    return true;
                }
            }
        }

        for &(df, dr) in &KNIGHT_JUMPS {
            let f = tf + df;
            let r = tr + dr;
            if f >= 0 && f < 8 && r >= 0 && r < 8 {
                if self.piece_at(sq(f as u8, r as u8)) == Some((by_color, Kind::Kingt)) {
                    return true;
                }
            }
        }

        for df in -1..=1i8 {
            for dr in -1..=1i8 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let f = tf + df;
                let r = tr + dr;
                if f >= 0 && f < 8 && r >= 0 && r < 8 {
                    if self.piece_at(sq(f as u8, r as u8)) == Some((by_color, Kind::King)) {
                        return true;
                    }
                }
            }
        }

        for &(df, dr) in &DIAGONALS {
            let mut f = tf + df;
            let mut r = tr + dr;
            while f >= 0 && f < 8 && r >= 0 && r < 8 {
                match self.piece_at(sq(f as u8, r as u8)) {
                    Some((c, k)) => {
                        if c == by_color && matches!(k, Kind::Bishop | Kind::Queen) {
                            return true;
                        }
                        break;
                    }
                    None => {}
                }
                f += df;
                r += dr;
            }
        }

        for &(df, dr) in &ORTHOGONALS {
            let mut f = tf + df;
            let mut r = tr + dr;
            while f >= 0 && f < 8 && r >= 0 && r < 8 {
                match self.piece_at(sq(f as u8, r as u8)) {
                    Some((c, k)) => {
                        if c == by_color && matches!(k, Kind::Rook | Kind::Queen) {
                            return true;
                        }
                        break;
                    }
                    None => {}
                }
                f += df;
                r += dr;
            }
        }

        false
    }

    fn gen_pawn_moves(&self, from: u8, moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        let f = file_of(from) as i8;
        let r = rank_of(from) as i8;
        let dir: i8 = match color {
            Color::White => 1,
            Color::Black => -1,
        };
        let start_rank: i8 = match color {
            Color::White => 1,
            Color::Black => 6,
        };
        let promo_rank: i8 = match color {
            Color::White => 7,
            Color::Black => 0,
        };

        let to_r = r + dir;
        if to_r >= 0 && to_r < 8 {
            let to_sq = sq(f as u8, to_r as u8);
            if self.piece_at(to_sq).is_none() {
                if to_r == promo_rank {
                    for kind in [Kind::Queen, Kind::Rook, Kind::Bishop, Kind::Kingt] {
                        moves.push(Move { from, to: to_sq, promotion: Some(kind) });
                    }
                } else {
                    moves.push(Move { from, to: to_sq, promotion: None });
                    if r == start_rank {
                        let to_sq2 = sq(f as u8, (r + 2 * dir) as u8);
                        if self.piece_at(to_sq2).is_none() {
                            moves.push(Move { from, to: to_sq2, promotion: None });
                        }
                    }
                }
            }

            for df in [-1i8, 1] {
                let cf = f + df;
                if cf < 0 || cf >= 8 {
                    continue;
                }
                let cap_sq = sq(cf as u8, to_r as u8);
                let is_capture = match self.piece_at(cap_sq) {
                    Some((c, _)) => c != color,
                    None => Some(cap_sq) == self.en_passant,
                };
                if is_capture {
                    if to_r == promo_rank {
                        for kind in [Kind::Queen, Kind::Rook, Kind::Bishop, Kind::Kingt] {
                            moves.push(Move { from, to: cap_sq, promotion: Some(kind) });
                        }
                    } else {
                        moves.push(Move { from, to: cap_sq, promotion: None });
                    }
                }
            }
        }
    }

    fn gen_knight_moves(&self, from: u8, moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        let f = file_of(from) as i8;
        let r = rank_of(from) as i8;
        for &(df, dr) in &KNIGHT_JUMPS {
            let tf = f + df;
            let tr = r + dr;
            if tf < 0 || tf >= 8 || tr < 0 || tr >= 8 {
                continue;
            }
            let to_sq = sq(tf as u8, tr as u8);
            match self.piece_at(to_sq) {
                Some((c, _)) if c == color => continue,
                _ => moves.push(Move { from, to: to_sq, promotion: None }),
            }
        }
    }

    fn gen_sliding_moves(&self, from: u8, directions: &[(i8, i8)], moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        let f = file_of(from) as i8;
        let r = rank_of(from) as i8;
        for &(df, dr) in directions {
            let mut tf = f + df;
            let mut tr = r + dr;
            while tf >= 0 && tf < 8 && tr >= 0 && tr < 8 {
                let to_sq = sq(tf as u8, tr as u8);
                match self.piece_at(to_sq) {
                    Some((c, _)) => {
                        if c != color {
                            moves.push(Move { from, to: to_sq, promotion: None });
                        }
                        break;
                    }
                    None => moves.push(Move { from, to: to_sq, promotion: None }),
                }
                tf += df;
                tr += dr;
            }
        }
    }

    fn gen_king_moves(&self, from: u8, moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        let f = file_of(from) as i8;
        let r = rank_of(from) as i8;

        for df in -1..=1i8 {
            for dr in -1..=1i8 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let tf = f + df;
                let tr = r + dr;
                if tf < 0 || tf >= 8 || tr < 0 || tr >= 8 {
                    continue;
                }
                let to_sq = sq(tf as u8, tr as u8);
                match self.piece_at(to_sq) {
                    Some((c, _)) if c == color => continue,
                    _ => moves.push(Move { from, to: to_sq, promotion: None }),
                }
            }
        }

        let opp = color.opposite();
        match color {
            Color::White => {
                if self.castling.white_kingside
                    && self.piece_at(sq(5, 0)).is_none()
                    && self.piece_at(sq(6, 0)).is_none()
                    && !self.is_attacked(sq(4, 0), opp)
                    && !self.is_attacked(sq(5, 0), opp)
                    && !self.is_attacked(sq(6, 0), opp)
                {
                    moves.push(Move { from, to: sq(6, 0), promotion: None });
                }
                if self.castling.white_queenside
                    && self.piece_at(sq(3, 0)).is_none()
                    && self.piece_at(sq(2, 0)).is_none()
                    && self.piece_at(sq(1, 0)).is_none()
                    && !self.is_attacked(sq(4, 0), opp)
                    && !self.is_attacked(sq(3, 0), opp)
                    && !self.is_attacked(sq(2, 0), opp)
                {
                    moves.push(Move { from, to: sq(2, 0), promotion: None });
                }
            }
            Color::Black => {
                if self.castling.black_kingside
                    && self.piece_at(sq(5, 7)).is_none()
                    && self.piece_at(sq(6, 7)).is_none()
                    && !self.is_attacked(sq(4, 7), opp)
                    && !self.is_attacked(sq(5, 7), opp)
                    && !self.is_attacked(sq(6, 7), opp)
                {
                    moves.push(Move { from, to: sq(6, 7), promotion: None });
                }
                if self.castling.black_queenside
                    && self.piece_at(sq(3, 7)).is_none()
                    && self.piece_at(sq(2, 7)).is_none()
                    && self.piece_at(sq(1, 7)).is_none()
                    && !self.is_attacked(sq(4, 7), opp)
                    && !self.is_attacked(sq(3, 7), opp)
                    && !self.is_attacked(sq(2, 7), opp)
                {
                    moves.push(Move { from, to: sq(2, 7), promotion: None });
                }
            }
        }
    }

    fn update_castling_for_square(&mut self, sq: u8) {
        match sq {
            0 => self.castling.white_queenside = false,
            7 => self.castling.white_kingside = false,
            56 => self.castling.black_queenside = false,
            63 => self.castling.black_kingside = false,
            _ => {}
        }
    }
}

impl Board for Mailbox {
    fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    fn piece_at(&self, sq: u8) -> Option<Piece> {
        self.squares[sq as usize]
    }

    fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(format!("expected 6 FEN fields, got {}", parts.len()));
        }

        let mut board = Self::empty();

        let mut rank: u8 = 7;
        let mut file: u8 = 0;
        for ch in parts[0].chars() {
            match ch {
                '/' => {
                    if file != 8 {
                        return Err(format!("rank {} has {} files", rank + 1, file));
                    }
                    rank = rank.checked_sub(1).ok_or("too many ranks")?;
                    file = 0;
                }
                '1'..='8' => {
                    file += ch as u8 - b'0';
                }
                _ => {
                    let piece = char_to_piece(ch)?;
                    board.set_piece(sq(file, rank), Some(piece));
                    file += 1;
                }
            }
        }

        board.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            other => return Err(format!("invalid side: {}", other)),
        };

        if parts[2] != "-" {
            for ch in parts[2].chars() {
                match ch {
                    'K' => board.castling.white_kingside = true,
                    'Q' => board.castling.white_queenside = true,
                    'k' => board.castling.black_kingside = true,
                    'q' => board.castling.black_queenside = true,
                    other => return Err(format!("invalid castling char: {}", other)),
                }
            }
        }

        if parts[3] != "-" {
            board.en_passant = Some(parse_square(parts[3])?);
        }

        board.halfmove_clock = parts[4].parse().map_err(|_| "invalid halfmove clock")?;
        board.fullmove_number = parts[5].parse().map_err(|_| "invalid fullmove number")?;

        Ok(board)
    }

    fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8u8).rev() {
            let mut empty = 0u8;
            for file in 0..8u8 {
                match self.piece_at(sq(file, rank)) {
                    Some(piece) => {
                        if empty > 0 {
                            fen.push((b'0' + empty) as char);
                            empty = 0;
                        }
                        fen.push(piece_to_char(piece));
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                fen.push((b'0' + empty) as char);
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        fen.push(' ');
        let mut any = false;
        if self.castling.white_kingside { fen.push('K'); any = true; }
        if self.castling.white_queenside { fen.push('Q'); any = true; }
        if self.castling.black_kingside { fen.push('k'); any = true; }
        if self.castling.black_queenside { fen.push('q'); any = true; }
        if !any { fen.push('-'); }

        fen.push(' ');
        match self.en_passant {
            Some(ep) => {
                fen.push((b'a' + file_of(ep)) as char);
                fen.push((b'1' + rank_of(ep)) as char);
            }
            None => fen.push('-'),
        }

        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    fn make_move(&mut self, mov: Move) {
        let from = mov.from;
        let to = mov.to;
        let promotion = mov.promotion;

        let piece = match self.piece_at(from) {
            Some(p) => p,
            None => return,
        };
        let (color, kind) = piece;
        let is_pawn = matches!(kind, Kind::Pawn);
        let is_capture = self.piece_at(to).is_some()
            || (is_pawn && Some(to) == self.en_passant);

        if is_pawn && Some(to) == self.en_passant {
            let captured_sq = match color {
                Color::White => to - 8,
                Color::Black => to + 8,
            };
            self.set_piece(captured_sq, None);
        }

        self.set_piece(from, None);
        let placed = match promotion {
            Some(promo_kind) => (color, promo_kind),
            None => piece,
        };
        self.set_piece(to, Some(placed));

        if matches!(kind, Kind::King) {
            let diff = to as i8 - from as i8;
            if diff == 2 {
                let rook = self.piece_at(from + 3);
                self.set_piece(from + 3, None);
                self.set_piece(from + 1, rook);
            } else if diff == -2 {
                let rook = self.piece_at(from - 4);
                self.set_piece(from - 4, None);
                self.set_piece(from - 1, rook);
            }
        }

        self.en_passant = None;
        if is_pawn {
            let diff = (to as i8 - from as i8).unsigned_abs();
            if diff == 16 {
                self.en_passant = Some((from + to) / 2);
            }
        }

        if matches!(kind, Kind::King) {
            match color {
                Color::White => {
                    self.castling.white_kingside = false;
                    self.castling.white_queenside = false;
                }
                Color::Black => {
                    self.castling.black_kingside = false;
                    self.castling.black_queenside = false;
                }
            }
        }
        self.update_castling_for_square(from);
        self.update_castling_for_square(to);

        if is_pawn || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if matches!(color, Color::Black) {
            self.fullmove_number += 1;
        }

        self.side_to_move = self.side_to_move.opposite();
    }

    fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let us = self.side_to_move;

        for i in 0..64u8 {
            if let Some((color, kind)) = self.piece_at(i) {
                if color != us {
                    continue;
                }
                match kind {
                    Kind::Pawn => self.gen_pawn_moves(i, &mut moves),
                    Kind::Kingt => self.gen_knight_moves(i, &mut moves),
                    Kind::Bishop => self.gen_sliding_moves(i, &DIAGONALS, &mut moves),
                    Kind::Rook => self.gen_sliding_moves(i, &ORTHOGONALS, &mut moves),
                    Kind::Queen => {
                        self.gen_sliding_moves(i, &DIAGONALS, &mut moves);
                        self.gen_sliding_moves(i, &ORTHOGONALS, &mut moves);
                    }
                    Kind::King => self.gen_king_moves(i, &mut moves),
                }
            }
        }

        moves.into_iter().filter(|m| {
            let mut copy = self.clone();
            copy.make_move(*m);
            !copy.is_in_check(us)
        }).collect()
    }

    fn is_in_check(&self, color: Color) -> bool {
        match self.find_king(color) {
            Some(king_sq) => self.is_attacked(king_sq, color.opposite()),
            None => false,
        }
    }
}

pub fn perft(board: &Mailbox, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = board.generate_moves();
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut count = 0u64;
    for m in moves {
        let mut copy = board.clone();
        copy.make_move(m);
        count += perft(&copy, depth - 1);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    fn mv(from: &str, to: &str) -> Move {
        Move {
            from: parse_square(from).unwrap(),
            to: parse_square(to).unwrap(),
            promotion: None,
        }
    }

    fn mvp(from: &str, to: &str, promo: Kind) -> Move {
        Move {
            from: parse_square(from).unwrap(),
            to: parse_square(to).unwrap(),
            promotion: Some(promo),
        }
    }

    fn has_move(moves: &[Move], m: Move) -> bool {
        moves.contains(&m)
    }

    #[test]
    fn empty_board_fen() {
        let board = Mailbox::empty();
        assert_eq!(board.to_fen(), "8/8/8/8/8/8/8/8 w - - 0 1");
    }

    #[test]
    fn startpos_fen_roundtrip() {
        let board = Mailbox::startpos();
        assert_eq!(board.to_fen(), STARTPOS);
    }

    #[test]
    fn fen_roundtrip_midgame() {
        let fen = "r1bqkb1r/pppppppp/2n2n2/8/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let board = Mailbox::from_fen(fen).unwrap();
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn fen_roundtrip_with_en_passant() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board = Mailbox::from_fen(fen).unwrap();
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn fen_roundtrip_no_castling() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1";
        let board = Mailbox::from_fen(fen).unwrap();
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn fen_invalid_field_count() {
        assert!(Mailbox::from_fen("8/8/8/8/8/8/8/8 w").is_err());
    }

    #[test]
    fn piece_at_startpos() {
        let board = Mailbox::startpos();
        assert_eq!(board.piece_at(sq(0, 0)), Some((Color::White, Kind::Rook)));
        assert_eq!(board.piece_at(sq(4, 0)), Some((Color::White, Kind::King)));
        assert_eq!(board.piece_at(sq(4, 7)), Some((Color::Black, Kind::King)));
        assert_eq!(board.piece_at(sq(0, 1)), Some((Color::White, Kind::Pawn)));
        assert_eq!(board.piece_at(sq(0, 6)), Some((Color::Black, Kind::Pawn)));
        assert_eq!(board.piece_at(sq(3, 3)), None);
    }

    #[test]
    fn make_move_e2e4() {
        let mut board = Mailbox::startpos();
        board.make_move(mv("e2", "e4"));
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    fn make_move_sequence_italian() {
        let mut board = Mailbox::startpos();
        for m in [mv("e2", "e4"), mv("e7", "e5"), mv("g1", "f3"), mv("b8", "c6")] {
            board.make_move(m);
        }
        assert_eq!(
            board.to_fen(),
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3"
        );
    }

    #[test]
    fn make_move_capture_resets_halfmove() {
        let mut board = Mailbox::from_fen(
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
        ).unwrap();
        board.make_move(mv("e4", "d5"));
        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.piece_at(parse_square("d5").unwrap()), Some((Color::White, Kind::Pawn)));
        assert_eq!(board.piece_at(parse_square("e4").unwrap()), None);
    }

    #[test]
    fn make_move_en_passant_white() {
        let mut board = Mailbox::from_fen(
            "rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3",
        ).unwrap();
        board.make_move(mv("f5", "e6"));
        assert_eq!(board.piece_at(parse_square("e6").unwrap()), Some((Color::White, Kind::Pawn)));
        assert_eq!(board.piece_at(parse_square("e5").unwrap()), None);
        assert_eq!(board.piece_at(parse_square("f5").unwrap()), None);
    }

    #[test]
    fn make_move_en_passant_black() {
        let mut board = Mailbox::from_fen(
            "rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3",
        ).unwrap();
        board.make_move(mv("f4", "e3"));
        assert_eq!(board.piece_at(parse_square("e3").unwrap()), Some((Color::Black, Kind::Pawn)));
        assert_eq!(board.piece_at(parse_square("e4").unwrap()), None);
        assert_eq!(board.piece_at(parse_square("f4").unwrap()), None);
    }

    #[test]
    fn make_move_kingside_castle_white() {
        let mut board = Mailbox::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1",
        ).unwrap();
        board.make_move(mv("e1", "g1"));
        assert_eq!(board.piece_at(parse_square("g1").unwrap()), Some((Color::White, Kind::King)));
        assert_eq!(board.piece_at(parse_square("f1").unwrap()), Some((Color::White, Kind::Rook)));
        assert_eq!(board.piece_at(parse_square("e1").unwrap()), None);
        assert_eq!(board.piece_at(parse_square("h1").unwrap()), None);
        assert!(!board.castling.white_kingside);
        assert!(!board.castling.white_queenside);
    }

    #[test]
    fn make_move_queenside_castle_white() {
        let mut board = Mailbox::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3KBNR w KQkq - 0 1",
        ).unwrap();
        board.make_move(mv("e1", "c1"));
        assert_eq!(board.piece_at(parse_square("c1").unwrap()), Some((Color::White, Kind::King)));
        assert_eq!(board.piece_at(parse_square("d1").unwrap()), Some((Color::White, Kind::Rook)));
        assert_eq!(board.piece_at(parse_square("e1").unwrap()), None);
        assert_eq!(board.piece_at(parse_square("a1").unwrap()), None);
    }

    #[test]
    fn make_move_kingside_castle_black() {
        let mut board = Mailbox::from_fen(
            "rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        ).unwrap();
        board.make_move(mv("e8", "g8"));
        assert_eq!(board.piece_at(parse_square("g8").unwrap()), Some((Color::Black, Kind::King)));
        assert_eq!(board.piece_at(parse_square("f8").unwrap()), Some((Color::Black, Kind::Rook)));
        assert_eq!(board.piece_at(parse_square("e8").unwrap()), None);
        assert_eq!(board.piece_at(parse_square("h8").unwrap()), None);
        assert!(!board.castling.black_kingside);
        assert!(!board.castling.black_queenside);
    }

    #[test]
    fn make_move_promotion() {
        let mut board = Mailbox::from_fen(
            "8/4P3/8/8/8/8/8/4K2k w - - 0 1",
        ).unwrap();
        board.make_move(mvp("e7", "e8", Kind::Queen));
        assert_eq!(board.piece_at(parse_square("e8").unwrap()), Some((Color::White, Kind::Queen)));
        assert_eq!(board.piece_at(parse_square("e7").unwrap()), None);
    }

    #[test]
    fn make_move_promotion_capture() {
        let mut board = Mailbox::from_fen(
            "3r4/4P3/8/8/8/8/8/4K2k w - - 0 1",
        ).unwrap();
        board.make_move(mvp("e7", "d8", Kind::Kingt));
        assert_eq!(board.piece_at(parse_square("d8").unwrap()), Some((Color::White, Kind::Kingt)));
    }

    #[test]
    fn make_move_rook_move_removes_castling() {
        let mut board = Mailbox::from_fen(
            "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        ).unwrap();
        board.make_move(mv("h1", "h2"));
        assert!(!board.castling.white_kingside);
        assert!(board.castling.white_queenside);
    }

    #[test]
    fn make_move_rook_captured_removes_castling() {
        let mut board = Mailbox::from_fen(
            "r3k2r/pppppppp/8/8/8/7B/PPPPPPPP/R3K2R w KQkq - 0 1",
        ).unwrap();
        board.make_move(mv("h3", "a8"));
        assert!(!board.castling.black_queenside);
        assert!(board.castling.black_kingside);
    }

    #[test]
    fn make_move_fullmove_increments_after_black() {
        let mut board = Mailbox::startpos();
        board.make_move(mv("e2", "e4"));
        assert_eq!(board.fullmove_number, 1);
        board.make_move(mv("e7", "e5"));
        assert_eq!(board.fullmove_number, 2);
    }

    #[test]
    fn make_move_halfmove_increments_on_quiet() {
        let mut board = Mailbox::from_fen(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        ).unwrap();
        board.make_move(mv("b8", "c6"));
        assert_eq!(board.halfmove_clock, 1);
    }

    #[test]
    fn double_push_sets_en_passant() {
        let mut board = Mailbox::startpos();
        board.make_move(mv("d2", "d4"));
        assert_eq!(board.en_passant, Some(parse_square("d3").unwrap()));
    }

    #[test]
    fn en_passant_cleared_after_next_move() {
        let mut board = Mailbox::startpos();
        board.make_move(mv("e2", "e4"));
        assert!(board.en_passant.is_some());
        board.make_move(mv("b8", "c6"));
        assert_eq!(board.en_passant, None);
    }

    #[test]
    fn sq_helpers() {
        assert_eq!(sq(0, 0), 0);
        assert_eq!(sq(7, 7), 63);
        assert_eq!(sq(4, 3), 28);
        assert_eq!(file_of(28), 4);
        assert_eq!(rank_of(28), 3);
    }

    #[test]
    fn parse_square_valid() {
        assert_eq!(parse_square("a1").unwrap(), 0);
        assert_eq!(parse_square("h8").unwrap(), 63);
        assert_eq!(parse_square("e4").unwrap(), 28);
    }

    #[test]
    fn parse_square_invalid() {
        assert!(parse_square("i1").is_err());
        assert!(parse_square("a9").is_err());
        assert!(parse_square("abc").is_err());
    }

    #[test]
    fn is_in_check_startpos() {
        let board = Mailbox::startpos();
        assert!(!board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn is_in_check_fools_mate() {
        let board = Mailbox::from_fen(
            "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
        ).unwrap();
        assert!(board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn generate_moves_startpos_count() {
        let board = Mailbox::startpos();
        let moves = board.generate_moves();
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn generate_moves_includes_castling() {
        let board = Mailbox::from_fen(
            "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(has_move(&moves, mv("e1", "g1")));
        assert!(has_move(&moves, mv("e1", "c1")));
    }

    #[test]
    fn generate_moves_no_castling_through_check() {
        let board = Mailbox::from_fen(
            "4k3/8/8/8/5r2/8/8/R3K2R w KQ - 0 1",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(!has_move(&moves, mv("e1", "g1")));
        assert!(has_move(&moves, mv("e1", "c1")));
    }

    #[test]
    fn generate_moves_no_castling_in_check() {
        let board = Mailbox::from_fen(
            "4k3/8/8/8/4r3/8/8/R3K2R w KQ - 0 1",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(!has_move(&moves, mv("e1", "g1")));
        assert!(!has_move(&moves, mv("e1", "c1")));
    }

    #[test]
    fn generate_moves_en_passant() {
        let board = Mailbox::from_fen(
            "rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(has_move(&moves, mv("f5", "e6")));
    }

    #[test]
    fn generate_moves_promotion() {
        let board = Mailbox::from_fen(
            "8/4P3/8/8/8/8/8/4K2k w - - 0 1",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(has_move(&moves, mvp("e7", "e8", Kind::Queen)));
        assert!(has_move(&moves, mvp("e7", "e8", Kind::Rook)));
        assert!(has_move(&moves, mvp("e7", "e8", Kind::Bishop)));
        assert!(has_move(&moves, mvp("e7", "e8", Kind::Kingt)));
    }

    #[test]
    fn generate_moves_pinned_piece() {
        let board = Mailbox::from_fen(
            "8/8/8/8/8/1b6/2P5/3K3k w - - 0 1",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(!has_move(&moves, mv("c2", "c3")));
        assert!(!has_move(&moves, mv("c2", "c4")));
    }

    #[test]
    fn generate_moves_checkmate_no_moves() {
        let board = Mailbox::from_fen(
            "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(moves.is_empty());
    }

    #[test]
    fn generate_moves_stalemate_no_moves() {
        let board = Mailbox::from_fen(
            "7k/8/6QK/8/8/8/8/8 b - - 0 1",
        ).unwrap();
        let moves = board.generate_moves();
        assert!(moves.is_empty());
    }

    #[test]
    fn perft_startpos_depth1() {
        let board = Mailbox::startpos();
        assert_eq!(perft(&board, 1), 20);
    }

    #[test]
    fn perft_startpos_depth2() {
        let board = Mailbox::startpos();
        assert_eq!(perft(&board, 2), 400);
    }

    #[test]
    fn perft_startpos_depth3() {
        let board = Mailbox::startpos();
        assert_eq!(perft(&board, 3), 8902);
    }

    #[test]
    fn perft_startpos_depth4() {
        let board = Mailbox::startpos();
        assert_eq!(perft(&board, 4), 197281);
    }

    #[test]
    fn perft_kiwipete_depth1() {
        let board = Mailbox::from_fen(KIWIPETE).unwrap();
        assert_eq!(perft(&board, 1), 48);
    }

    #[test]
    fn perft_kiwipete_depth2() {
        let board = Mailbox::from_fen(KIWIPETE).unwrap();
        assert_eq!(perft(&board, 2), 2039);
    }

    #[test]
    fn perft_kiwipete_depth3() {
        let board = Mailbox::from_fen(KIWIPETE).unwrap();
        assert_eq!(perft(&board, 3), 97862);
    }
}
