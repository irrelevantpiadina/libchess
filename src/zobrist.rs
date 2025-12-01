use rand::Rng;

use crate::{ZobristValues, color, piece::bb, pos};

pub type Key = u64;

/// used to generate all random values needed to create a zobrist key
pub(crate) fn init_zb_values(zb: &mut ZobristValues) {
    let mut rng = rand::rng();

    zb.black_to_move = rng.random();
    zb.wk_castle = rng.random();
    zb.wq_castle = rng.random();
    zb.bk_castle = rng.random();
    zb.bq_castle = rng.random();

    for file in zb.ep_files.iter_mut() {
        *file = rng.random();
    }

    for piece in zb.piece_sq.iter_mut() {
        for sq in piece.iter_mut() {
            *sq = rng.random();
        }
    }
}

/// creates and returns a zobrist key for `pos`
///
/// any positions that are equal to each other will generate the same key
///
/// the function takes into account the locations and types of all pieces on the board,
/// the side to move, en passant files, and castling rights
pub fn hash(pos: &pos::Position, zb: &ZobristValues) -> Key {
    let mut key = 0;

    for sq in 0..64 {
        if pos.is_occupied(sq) {
            key ^= zb.piece_sq[bb::p_to_idx(pos.piece_on(sq))][sq as pos::Square];
        }
    }

    if pos.side_to_move() == color::BLACK {
        key ^= zb.black_to_move;
    }

    if let Some(square) = pos.ep_square() {
        key ^= zb.ep_files[pos::file_of(square) as usize];
    }

    if pos.castle_rights() & pos::WK_CASTLE != 0 {
        key ^= zb.wk_castle;
    }

    if pos.castle_rights() & pos::WQ_CASTLE != 0 {
        key ^= zb.wq_castle;
    }

    if pos.castle_rights() & pos::BK_CASTLE != 0 {
        key ^= zb.bk_castle;
    }

    if pos.castle_rights() & pos::BQ_CASTLE != 0 {
        key ^= zb.bq_castle;
    }

    key
}
