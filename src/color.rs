use crate::piece;

pub type Color = u8;

pub const NONE: Color = 0x0;
pub const WHITE: Color = 0x40;
pub const BLACK: Color = 0x80;

pub const MASK: Color = WHITE | BLACK;

/// swtiches `color` to `WHITE` if previously `BLACK`, and vice versa,
///
/// returns the switched value of `color`
#[inline(always)]
pub fn switch(color: &mut Color) -> Color {
    *color ^= MASK;

    *color
}

/// returns `WHITE` if `color` is `BLACK`, and vice versa
#[inline(always)]
pub fn other(color: Color) -> Color {
    color ^ MASK
}

/// returns the color of `piece`, if any
#[inline(always)]
pub fn of(piece: piece::Piece) -> Color {
    piece & MASK
}

#[inline(always)]
pub fn to_str(color: Color) -> &'static str {
    match color {
        WHITE => "white",
        BLACK => "black",
        NONE => "no color",
        _ => "unknown",
    }
}
