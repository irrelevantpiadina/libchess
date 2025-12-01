use crate::piece::bb::{self, BitboardUtil};

pub mod color;
pub mod moves;
pub mod perft;
pub mod piece;
pub mod pos;
pub mod uci;
pub mod zobrist;

#[derive(Debug, Clone)]
/// attack masks for all pieces on all squares
pub struct AttackMasks {
    pawn_attacks: [[bb::Bitboard; 64]; 2],
    knight_attacks: [bb::Bitboard; 64],
    king_attacks: [bb::Bitboard; 64],
    rook_rays: [bb::Bitboard; 64],
    bishop_rays: [bb::Bitboard; 64],
}

#[derive(Debug, Clone)]
/// values used to generate position keys for transposition tables
pub struct ZobristValues {
    black_to_move: u64,
    wk_castle: u64,
    wq_castle: u64,
    bk_castle: u64,
    bq_castle: u64,
    ep_files: [u64; 8],
    piece_sq: [[u64; 64]; 12],
}

/// initializes lookup tables of attack masks necessary for move generation,
/// and zobrist values needed for generating position keys
pub fn init() -> (AttackMasks, ZobristValues) {
    let mut masks = AttackMasks {
        pawn_attacks: [[bb::EMPTY; 64]; 2],
        knight_attacks: [bb::EMPTY; 64],
        king_attacks: [bb::EMPTY; 64],
        rook_rays: [bb::EMPTY; 64],
        bishop_rays: [bb::EMPTY; 64],
    };

    bb::init_attack_masks_non_sliding_piece(&mut masks);
    bb::init_attack_masks_sliding_piece_rays(&mut masks);

    let mut zb = ZobristValues {
        black_to_move: 0,
        wk_castle: 0,
        wq_castle: 0,
        bk_castle: 0,
        bq_castle: 0,
        ep_files: [0; 8],
        piece_sq: [[0; 64]; 12],
    };

    zobrist::init_zb_values(&mut zb);

    (masks, zb)
}

impl AttackMasks {
    #[inline(always)]
    pub fn pawn_attacks(&self, color: color::Color, square: pos::Square) -> bb::Bitboard {
        self.pawn_attacks[match color {
            color::WHITE => 0,
            _ => 1,
        }][square]
    }

    #[inline(always)]
    pub fn knight_attacks(&self, square: pos::Square) -> bb::Bitboard {
        self.knight_attacks[square]
    }

    #[inline(always)]
    pub fn king_attacks(&self, square: pos::Square) -> bb::Bitboard {
        self.king_attacks[square]
    }

    #[inline(always)]
    pub fn rook_rays(&self, square: pos::Square) -> bb::Bitboard {
        self.rook_rays[square]
    }

    #[inline(always)]
    pub fn bishop_rays(&self, square: pos::Square) -> bb::Bitboard {
        self.bishop_rays[square]
    }

    #[inline(always)]
    pub fn queen_rays(&self, square: pos::Square) -> bb::Bitboard {
        self.rook_rays[square] | self.bishop_rays[square]
    }

    pub fn rook_attacks_rt(&self, square: pos::Square, occupied: bb::Bitboard) -> bb::Bitboard {
        let blockers = occupied & self.rook_rays(square);
        (bb::walk_to_blocker(square as isize, blockers, bb::RANK_8_MASK, 8)
            | bb::walk_to_blocker(square as isize, blockers, bb::RANK_1_MASK, -8)
            | bb::walk_to_blocker(square as isize, blockers, bb::FILE_H_MASK, 1)
            | bb::walk_to_blocker(square as isize, blockers, bb::FILE_A_MASK, -1))
        .pop_bit(square)
    }

    pub fn bishop_attacks_rt(&self, square: pos::Square, occupied: bb::Bitboard) -> bb::Bitboard {
        let blockers = occupied & self.bishop_rays(square);
        (bb::walk_to_blocker(
            square as isize,
            blockers,
            bb::FILE_A_MASK | bb::RANK_8_MASK,
            7,
        ) | bb::walk_to_blocker(
            square as isize,
            blockers,
            bb::FILE_H_MASK | bb::RANK_1_MASK,
            -7,
        ) | bb::walk_to_blocker(
            square as isize,
            blockers,
            bb::FILE_H_MASK | bb::RANK_8_MASK,
            9,
        ) | bb::walk_to_blocker(
            square as isize,
            blockers,
            bb::FILE_A_MASK | bb::RANK_1_MASK,
            -9,
        ))
        .pop_bit(square)
    }

    pub fn queen_attacks_rt(&self, square: pos::Square, occupied: bb::Bitboard) -> bb::Bitboard {
        self.rook_attacks_rt(square, occupied) | self.bishop_attacks_rt(square, occupied)
    }
}
