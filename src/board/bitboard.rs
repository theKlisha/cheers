
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
