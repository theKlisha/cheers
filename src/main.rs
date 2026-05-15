#![allow(dead_code)]

pub enum Move {
    White,
    Black,
}

pub enum Piece {
    Pawn,
    Rook,
    Kingt,
    Bishop,
    Queen,
    King,
}

pub struct BitBoard(u64);

pub struct Occupancy {
    pawn: BitBoard,
    rook: BitBoard,
    kingt: BitBoard,
    bishop: BitBoard,
    queen: BitBoard,
    king: BitBoard,
}

pub struct Board {
    white_occ: Occupancy,
    black_occ: Occupancy,
}

fn main() {
    println!("Hello, world!");
}
