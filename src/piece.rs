pub mod bb;

use crate::color;

pub type Piece = u8;

pub const NONE: Piece = 0x0;
pub const PAWN: Piece = 0x1;
pub const KNIGHT: Piece = 0x2;
pub const BISHOP: Piece = 0x4;
pub const ROOK: Piece = 0x8;
pub const QUEEN: Piece = 0x10;
pub const KING: Piece = 0x20;

pub const MASK: Piece = PAWN | KNIGHT | BISHOP | ROOK | QUEEN | KING;

pub const WHITE_PAWN: Piece = PAWN | color::WHITE;
pub const WHITE_KNIGHT: Piece = KNIGHT | color::WHITE;
pub const WHITE_BISHOP: Piece = BISHOP | color::WHITE;
pub const WHITE_ROOK: Piece = ROOK | color::WHITE;
pub const WHITE_QUEEN: Piece = QUEEN | color::WHITE;
pub const WHITE_KING: Piece = KING | color::WHITE;
pub const BLACK_PAWN: Piece = PAWN | color::BLACK;
pub const BLACK_KNIGHT: Piece = KNIGHT | color::BLACK;
pub const BLACK_BISHOP: Piece = BISHOP | color::BLACK;
pub const BLACK_ROOK: Piece = ROOK | color::BLACK;
pub const BLACK_QUEEN: Piece = QUEEN | color::BLACK;
pub const BLACK_KING: Piece = KING | color::BLACK;
pub const SLIDING_PIECE: Piece = ROOK | BISHOP | QUEEN;

/// returns the piece equivalent to `ch`, uppercase characters indicate white piece,
/// while lowercase characters indicate black pieces,
///
/// if there is no piece equivalent to `ch`, returns `NONE`
///
/// `e.g. 'p' => BLACK_PAWN`
pub fn from_char(ch: char) -> Piece {
    let mut piece = match ch.to_ascii_lowercase() {
        'p' => PAWN,
        'n' => KNIGHT,
        'b' => BISHOP,
        'r' => ROOK,
        'q' => QUEEN,
        'k' => KING,
        _ => NONE,
    };

    if ch.is_uppercase() && piece != NONE {
        piece |= color::WHITE;
    } else {
        piece |= color::BLACK;
    }

    piece
}

/// returns the character equivalent of `piece`, white pieces return uppercase characters,
/// while black pieces return lowercase characters
///
/// if `piece` is `NONE` or invalid, returns `' '`
///
/// `e.g. BLACK_PAWN => 'p'`
pub fn as_char(piece: Piece) -> char {
    let mut ch: char = ' ';

    if piece & PAWN != 0 {
        ch = 'p';
    } else if piece & KNIGHT != 0 {
        ch = 'n';
    } else if piece & BISHOP != 0 {
        ch = 'b';
    } else if piece & ROOK != 0 {
        ch = 'r';
    } else if piece & QUEEN != 0 {
        ch = 'q';
    } else if piece & KING != 0 {
        ch = 'k';
    }

    if piece & color::WHITE != 0 {
        ch.to_ascii_uppercase()
    } else {
        ch
    }
}

/// returns a UTF-8 symbol for each piece,
/// may not display properly if your font doesn't support them
///
/// if `piece` is `NONE` or invalid, returns `' '`
pub fn as_symbol(piece: Piece) -> &'static str {
    match piece {
        WHITE_PAWN => "♙",
        WHITE_KNIGHT => "♘",
        WHITE_BISHOP => "♗",
        WHITE_ROOK => "♖",
        WHITE_QUEEN => "♕",
        WHITE_KING => "♔",
        BLACK_PAWN => "♟",
        BLACK_KNIGHT => "♞",
        BLACK_BISHOP => "♝",
        BLACK_ROOK => "♜",
        BLACK_QUEEN => "♛",
        BLACK_KING => "♚",
        _ => " ",
    }
}

/// returns true if `piece` is any piece in `pieces`, otherwise false
#[inline(always)]
pub fn is_either(piece: Piece, pieces: &Vec<Piece>) -> bool {
    for &p in pieces {
        if piece & p != 0 {
            return true;
        }
    }

    false
}

/// returns the piece flag of `piece`, ignoring any color flags
#[inline(always)]
pub fn of(piece: Piece) -> Piece {
    piece & MASK
}
