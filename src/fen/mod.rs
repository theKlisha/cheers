use std::fmt;

#[cfg(test)]
mod tests;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{one_of, space1, u32 as parse_u32},
    combinator::{map, value},
    multi::{many1, separated_list1},
    sequence::pair,
};

use crate::{CastlingRights, Color, Kind, Piece, Square};

fn piece_char(piece: Piece) -> char {
    let ch = match piece.1 {
        Kind::Pawn => 'p',
        Kind::Rook => 'r',
        Kind::Knight => 'n',
        Kind::Bishop => 'b',
        Kind::Queen => 'q',
        Kind::King => 'k',
    };
    match piece.0 {
        Color::White => ch.to_ascii_uppercase(),
        Color::Black => ch,
    }
}

fn fen_piece(i: &str) -> IResult<&str, Piece> {
    map(one_of("pPrRnNbBqQkK"), |c: char| {
        let color = if c.is_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let kind = match c.to_ascii_lowercase() {
            'p' => Kind::Pawn,
            'r' => Kind::Rook,
            'n' => Kind::Knight,
            'b' => Kind::Bishop,
            'q' => Kind::Queen,
            'k' => Kind::King,
            _ => unreachable!(),
        };
        (color, kind)
    })
    .parse(i)
}

fn fen_rank(i: &str) -> IResult<&str, [Option<Piece>; 8]> {
    let (i, tokens) = many1(alt((
        map(fen_piece, |p| vec![Some(p)]),
        map(one_of("12345678"), |c: char| {
            vec![None; c as usize - '0' as usize]
        }),
    )))
    .parse(i)?;
    let flat: Vec<Option<Piece>> = tokens.into_iter().flatten().collect();
    if flat.len() != 8 {
        return Err(nom::Err::Error(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Verify,
        )));
    }
    let mut arr = [None; 8];
    arr.copy_from_slice(&flat);
    Ok((i, arr))
}

fn fen_pieces(i: &str) -> IResult<&str, [Option<Piece>; 64]> {
    let (i, ranks) = separated_list1(tag("/"), fen_rank).parse(i)?;
    if ranks.len() != 8 {
        return Err(nom::Err::Error(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Verify,
        )));
    }
    let mut squares = [None; 64];
    for (rank_idx, rank) in ranks.into_iter().enumerate() {
        let actual_rank = 7 - rank_idx as u8;
        for (file, piece) in rank.iter().enumerate() {
            squares[(actual_rank * 8 + file as u8) as usize] = *piece;
        }
    }
    Ok((i, squares))
}

fn fen_castling(i: &str) -> IResult<&str, CastlingRights> {
    alt((
        value(CastlingRights::none(), tag("-")),
        map(many1(one_of("KQkq")), |chars| {
            let mut rights = CastlingRights::none();
            for c in chars {
                match c {
                    'K' => rights.white_kingside = true,
                    'Q' => rights.white_queenside = true,
                    'k' => rights.black_kingside = true,
                    'q' => rights.black_queenside = true,
                    _ => unreachable!(),
                }
            }
            rights
        }),
    ))
    .parse(i)
}

fn fen_en_passant(i: &str) -> IResult<&str, Option<Square>> {
    alt((
        value(None, tag("-")),
        map(
            pair(one_of("abcdefgh"), one_of("12345678")),
            |(f, r)| Some(Square::from((r as u8 - b'1') * 8 + (f as u8 - b'a'))),
        ),
    ))
    .parse(i)
}

fn parse_fen_str(i: &str) -> IResult<&str, Fen> {
    let (i, squares) = fen_pieces(i)?;
    let (i, _) = space1(i)?;
    let (i, side_to_move) = alt((
        value(Color::White, tag("w")),
        value(Color::Black, tag("b")),
    ))
    .parse(i)?;
    let (i, _) = space1(i)?;
    let (i, castling) = fen_castling(i)?;
    let (i, _) = space1(i)?;
    let (i, en_passant) = fen_en_passant(i)?;
    let (i, _) = space1(i)?;
    let (i, halfmove_clock) = parse_u32(i)?;
    let (i, _) = space1(i)?;
    let (i, fullmove_number) = parse_u32(i)?;
    Ok((
        i,
        Fen {
            squares,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
        },
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fen {
    pub squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl TryFrom<&str> for Fen {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, String> {
        parse_fen_str(s)
            .map(|(_, fen)| fen)
            .map_err(|e| e.to_string())
    }
}

impl fmt::Display for Fen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8u8).rev() {
            let mut empty = 0u8;
            for file in 0..8u8 {
                match self.squares[(rank * 8 + file) as usize] {
                    Some(piece) => {
                        if empty > 0 {
                            write!(f, "{empty}")?;
                            empty = 0;
                        }
                        write!(f, "{}", piece_char(piece))?;
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                write!(f, "{empty}")?;
            }
            if rank > 0 {
                write!(f, "/")?;
            }
        }

        write!(
            f,
            " {} ",
            match self.side_to_move {
                Color::White => 'w',
                Color::Black => 'b',
            }
        )?;

        let mut any = false;
        if self.castling.white_kingside {
            write!(f, "K")?;
            any = true;
        }
        if self.castling.white_queenside {
            write!(f, "Q")?;
            any = true;
        }
        if self.castling.black_kingside {
            write!(f, "k")?;
            any = true;
        }
        if self.castling.black_queenside {
            write!(f, "q")?;
            any = true;
        }
        if !any {
            write!(f, "-")?;
        }

        write!(f, " ")?;
        match self.en_passant {
            Some(sq) => write!(
                f,
                "{}{}",
                (b'a' + sq.file as u8) as char,
                (b'1' + sq.rank as u8) as char
            )?,
            None => write!(f, "-")?,
        }

        write!(f, " {} {}", self.halfmove_clock, self.fullmove_number)
    }
}
