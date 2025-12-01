// file for all bitboard related stuff

use crate::{AttackMasks, color, piece, pos};

pub type Bitboard = u64;

pub const EMPTY: Bitboard = 0;

pub const FILE_A_MASK: Bitboard = 0x0101010101010101;
pub const FILE_H_MASK: Bitboard = 0x8080808080808080;
pub const FILE_AB_MASK: Bitboard = FILE_A_MASK | (FILE_A_MASK << 1);
pub const FILE_GH_MASK: Bitboard = FILE_H_MASK | (FILE_H_MASK >> 1);

pub const RANK_1_MASK: Bitboard = 0x00000000000000FF;
pub const RANK_8_MASK: Bitboard = 0xFF00000000000000;

pub const MAIN_DIAG_MASK: Bitboard = 0x8040201008040201;
pub const MAIN_ANTI_DIAG_MASK: Bitboard = 0x0102040810204080;

pub trait BitboardUtil {
    /// isolates the least signifcant 1 bit of `self`
    fn ls1b(self) -> Bitboard;

    /// visualize the bits of an integer as an 8x8 board
    ///
    /// the least significant bit is in the lower left corner,
    /// and the most significant bit in the upper right corner
    ///
    /// not the fastest implementation, but it works
    fn see_bits(self);

    /// takes a vector and appends the indices of all 1 bits in a bitboard,
    fn serialize_to_vec(self, vec: &mut Vec<usize>);

    /// returns an array of 64 elements containing the indices of all 1 bits in a bitboard,
    /// this function is faster than `bb::serialize_to_vec()` if you would otherwise have to create a new vector for every
    /// serialization,
    ///
    /// the function also returns the *actual* size of the array, aka how many elements were actually set by the function
    fn serialize_to_arr(self) -> ([usize; 64], usize);

    /// returns the index of the least significant 1 bit of a bitboard, and resets it
    ///
    /// therefore, calling this function repeatedly on the same bitboard will yield different results,
    /// as `self` is mutated,
    ///
    /// can be used as a faster alternative to both `bb::serialize_to_vec()` and `bb::serialize_to_arr()`
    /// if you don't need to use more than one index at a time, for example, when iterating over all
    /// indices, and the index of the previous iteration can be discarded
    fn serialize_once(&mut self) -> usize;

    /// sets the bit at a certain index to `1`
    fn set_bit(&mut self, idx: usize) -> Bitboard;

    /// sets the bit at a certain index to `0`
    fn pop_bit(&mut self, idx: usize) -> Bitboard;
}

/// returns the array index associated with each piece type,
///
/// you may use this to get the index at which a bitboard for a certain piece type may be found
/// in an array of bitboards
#[inline(always)]
pub fn p_to_idx(piece: piece::Piece) -> usize {
    piece.trailing_zeros() as usize
        + match color::of(piece) {
            color::BLACK => 6,
            _ => 0,
        }
}

/// returns the array index associated with each color, 0 for white, 1 for black,
///
/// you may use this to get the index at which a bitboard for all pieces of a certain color may be found
/// in an array of 'color' bitboards
#[inline(always)]
pub fn c_to_idx(color: color::Color) -> usize {
    match color {
        color::BLACK => 1,
        _ => 0,
    }
}

/// returns a mask where all the bits of the file that `square` resides on
/// are set to 1
#[inline(always)]
pub fn file_mask(square: pos::Square) -> Bitboard {
    FILE_A_MASK << (square & 7)
}

/// returns a mask where all the bits of the rank that `square` resides on
/// are set to 1
#[inline(always)]
pub fn rank_mask(square: pos::Square) -> Bitboard {
    RANK_1_MASK << (square & 56)
}

/// returns a mask where all the bits of the diagonal that `square` resides on
/// are set to 1
#[inline(always)]
pub fn diag_mask(square: pos::Square) -> Bitboard {
    let sq_isz = square as isize;
    let diag = (sq_isz & 7) - (sq_isz >> 3);
    if diag >= 0 {
        MAIN_DIAG_MASK >> (diag * 8)
    } else {
        MAIN_DIAG_MASK << (-diag * 8)
    }
}

/// returns a mask where all the bits of the anti-diagonal that `square` resides on
/// are set to 1
#[inline(always)]
pub fn anti_diag_mask(square: pos::Square) -> Bitboard {
    let sq_isz = square as isize;
    let diag = 7 - (sq_isz & 7) - (sq_isz >> 3);
    if diag >= 0 {
        MAIN_ANTI_DIAG_MASK >> (diag * 8)
    } else {
        MAIN_ANTI_DIAG_MASK << (-diag * 8)
    }
}

/// returns a bitboard where all bits of `bb` are shifted north by 1 square
#[inline(always)]
pub fn north(bb: Bitboard) -> Bitboard {
    bb << 8
}

/// returns a bitboard where all bits of `bb` are shifted south by 1 square
#[inline(always)]
pub fn south(bb: Bitboard) -> Bitboard {
    bb >> 8
}

/// returns a bitboard where all bits of `bb` are shifted east by 1 square
#[inline(always)]
pub fn east(bb: Bitboard) -> Bitboard {
    bb << 1
}

/// returns a bitboard where all bits of `bb` are shifted west by 1 square
#[inline(always)]
pub fn west(bb: Bitboard) -> Bitboard {
    bb >> 1
}

/// returns a bitboard where all bits of `bb` are shifted north-east by 1 square
#[inline(always)]
pub fn no_ea(bb: Bitboard) -> Bitboard {
    bb << 9
}

/// returns a bitboard where all bits of `bb` are shifted north-west by 1 square
#[inline(always)]
pub fn no_we(bb: Bitboard) -> Bitboard {
    bb << 7
}

/// returns a bitboard where all bits of `bb` are shifted south-east by 1 square
#[inline(always)]
pub fn so_ea(bb: Bitboard) -> Bitboard {
    bb >> 7
}

/// returns a bitboard where all bits of `bb` are shifted south-west by 1 square
#[inline(always)]
pub fn so_we(bb: Bitboard) -> Bitboard {
    bb >> 9
}

impl BitboardUtil for Bitboard {
    #[inline(always)]
    fn ls1b(self) -> Bitboard {
        self & !self.wrapping_sub(1)
    }

    fn see_bits(self) {
        let mut line = String::new();
        let mut rank = pos::RANK_8 + 1;

        println!("\ndecimal: {self}");
        println!("hex: {self:#018X}\n");

        for b in (0..64).rev() {
            if self & 1 << b != 0 {
                line.push_str("1 ");
            } else {
                line.push_str(". ");
            }

            if b % 8 == 0 {
                println!("{}   {rank}", line.chars().rev().collect::<String>());
                line.clear();
                rank -= 1;
            }
        }

        println!("\n a b c d e f g h")
    }

    #[inline(always)]
    fn serialize_to_vec(mut self, vec: &mut Vec<usize>) {
        while self != EMPTY {
            let ls1b = self.ls1b();
            vec.push(ls1b.trailing_zeros() as usize);
            self &= !ls1b;
        }
    }

    #[inline(always)]
    fn serialize_to_arr(mut self) -> ([usize; 64], usize) {
        let mut arr = [0; 64];
        let mut cnt = 0;

        while self != EMPTY {
            let ls1b = self.ls1b();
            arr[cnt] = ls1b.trailing_zeros() as usize;
            self &= !ls1b;
            cnt += 1;
        }

        (arr, cnt)
    }

    #[inline(always)]
    fn serialize_once(&mut self) -> usize {
        let ls1b = self.ls1b();
        *self &= !ls1b;
        ls1b.trailing_zeros() as usize
    }

    #[inline(always)]
    fn set_bit(&mut self, idx: usize) -> Bitboard {
        *self |= 1 << idx;
        *self
    }

    #[inline(always)]
    fn pop_bit(&mut self, idx: usize) -> Bitboard {
        *self &= !(1 << idx);
        *self
    }
}

/// creates a bitboard of blockers of an attack mask from an index,
/// used in generating lookup tables for sliding piece attacks
/// (currently unused)
pub fn blockers_from_idx(idx: usize, attacks: Bitboard) -> Bitboard {
    let mut blockers = EMPTY;

    let mut cpy = attacks;
    let mut i = 0;

    while cpy != EMPTY {
        let bit = (1 << i & idx != 0) as u64;
        blockers |= bit << cpy.serialize_once();
        i += 1;
    }

    blockers & !FILE_A_MASK & !FILE_H_MASK & !RANK_1_MASK & !RANK_8_MASK
}

/// sets all bits from the bit at the start index to the index of the first blocker to `1`
/// (or until `edges` are hit, if there are no blockers),
/// adding `dir` as an offset to the start index on every iteration,
///
/// e.g. dir => 8 == go up, 1 == go right, -1 == go left
#[inline(always)]
pub fn walk_to_blocker(start: isize, blockers: Bitboard, edges: Bitboard, dir: isize) -> Bitboard {
    let mut bb = EMPTY;

    if 1 << start & (blockers | edges) != 0 {
        1 << start
    } else {
        bb |= 1 << start | walk_to_blocker(start + dir, blockers, edges, dir);
        bb
    }
}

/// returns a bitboard of all pieces of `color` that attack a given square
#[inline(always)]
pub fn attackers_of(
    square: pos::Square,
    pos: &pos::Position,
    color: color::Color,
    masks: &AttackMasks,
) -> Bitboard {
    (masks.pawn_attacks(color::other(color), square) & pos.piece_bb(piece::PAWN | color))
        | (masks.knight_attacks(square) & pos.piece_bb(piece::KNIGHT | color))
        | (masks.king_attacks(square) & pos.piece_bb(piece::KING | color))
        | (masks.rook_attacks_rt(square, pos.occupied_bb())
            & (pos.piece_bb(piece::ROOK | color) | pos.piece_bb(piece::QUEEN | color)))
        | (masks.bishop_attacks_rt(square, pos.occupied_bb())
            & (pos.piece_bb(piece::BISHOP | color) | pos.piece_bb(piece::QUEEN | color)))
}

/// returns true if a square is attacked by any piece of `color`,
/// faster alternative to `bb::attackers_of` if you don't need to know where the attackers are
#[inline(always)]
pub fn is_attacked(
    square: pos::Square,
    pos: &pos::Position,
    color: color::Color,
    masks: &AttackMasks,
) -> bool {
    masks.pawn_attacks(color::other(color), square) & pos.piece_bb(piece::PAWN | color) != EMPTY
        || masks.knight_attacks(square) & pos.piece_bb(piece::KNIGHT | color) != EMPTY
        || masks.king_attacks(square) & pos.piece_bb(piece::KING | color) != EMPTY
        || masks.rook_attacks_rt(square, pos.occupied_bb())
            & (pos.piece_bb(piece::ROOK | color) | pos.piece_bb(piece::QUEEN | color))
            != EMPTY
        || masks.bishop_attacks_rt(square, pos.occupied_bb())
            & (pos.piece_bb(piece::BISHOP | color) | pos.piece_bb(piece::QUEEN | color))
            != EMPTY
}

/// returns true if the piece on `square` *might* be pinned to the king,
/// doesn't do a proper check to actually ensure a pin
///
/// used for filtering legal moves
#[inline(always)]
pub(crate) fn might_be_pinned(pos: &mut pos::Position, square: pos::Square) -> bool {
    let king_pos = pos
        .piece_bb(piece::KING | pos.side_to_move())
        .serialize_once();

    (file_mask(square) == file_mask(king_pos)
        && file_mask(square)
            & (pos.piece_bb(piece::ROOK | color::other(pos.side_to_move()))
                | pos.piece_bb(piece::QUEEN | color::other(pos.side_to_move())))
            != 0)
        || (rank_mask(square) == rank_mask(king_pos)
            && rank_mask(square)
                & (pos.piece_bb(piece::ROOK | color::other(pos.side_to_move()))
                    | pos.piece_bb(piece::QUEEN | color::other(pos.side_to_move())))
                != 0)
        || (diag_mask(square) == diag_mask(king_pos)
            && diag_mask(square)
                & (pos.piece_bb(piece::BISHOP | color::other(pos.side_to_move()))
                    | pos.piece_bb(piece::QUEEN | color::other(pos.side_to_move())))
                != 0)
        || (anti_diag_mask(square) == anti_diag_mask(king_pos)
            && anti_diag_mask(square)
                & (pos.piece_bb(piece::BISHOP | color::other(pos.side_to_move()))
                    | pos.piece_bb(piece::QUEEN | color::other(pos.side_to_move())))
                != 0)
}

/// used to initialize lookup tables for non sliding piece attacks, so we can look them up when needed
pub(crate) fn init_attack_masks_non_sliding_piece(masks: &mut AttackMasks) {
    for sq in 0..64 {
        let bb = 1 << sq;

        masks.pawn_attacks[0][sq] = no_we(bb & !FILE_A_MASK) | no_ea(bb & !FILE_H_MASK);
        masks.pawn_attacks[1][sq] = so_we(bb & !FILE_A_MASK) | so_ea(bb & !FILE_H_MASK);

        masks.knight_attacks[sq] = north(no_we(bb & !FILE_A_MASK))
            | north(no_ea(bb & !FILE_H_MASK))
            | south(so_we(bb & !FILE_A_MASK))
            | south(so_ea(bb & !FILE_H_MASK))
            | west(no_we(bb & !FILE_AB_MASK))
            | west(so_we(bb & !FILE_AB_MASK))
            | east(no_ea(bb & !FILE_GH_MASK))
            | east(so_ea(bb & !FILE_GH_MASK));

        masks.king_attacks[sq] = north(bb)
            | south(bb)
            | west(bb & !FILE_A_MASK)
            | east(bb & !FILE_H_MASK)
            | no_we(bb & !FILE_A_MASK)
            | no_ea(bb & !FILE_H_MASK)
            | so_we(bb & !FILE_A_MASK)
            | so_ea(bb & !FILE_H_MASK);
    }
}

/// initializes lookup tables for sliding piece attacks on an otherwise-empty-board
pub(crate) fn init_attack_masks_sliding_piece_rays(masks: &mut AttackMasks) {
    for sq in 0..64 {
        masks.rook_rays[sq] = (file_mask(sq) | rank_mask(sq)).pop_bit(sq);
        masks.bishop_rays[sq] = (diag_mask(sq) | anti_diag_mask(sq)).pop_bit(sq);
    }
}
