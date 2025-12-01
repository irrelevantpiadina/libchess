use crate::{
    AttackMasks, ZobristValues, color,
    piece::{
        self,
        bb::{self, BitboardUtil, EMPTY},
    },
    pos,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// all types of moves you can play,
/// we need to differentiate between these when making and unmaking moves
pub enum MoveType {
    Normal,
    PawnTwoUp,
    Capture(piece::Piece),
    Promotion(piece::Piece),
    PromoCapture(
        piece::Piece, /* promotion */
        piece::Piece, /* capture */
    ),
    EnPassant,
    KingSideCastle,
    QueenSideCastle,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// a move
pub struct Move {
    from_sq: pos::Square,
    to_sq: pos::Square,
    type_of: MoveType,
    pub(crate) is_reversible: bool,
}

impl Move {
    /// ***panics*** on debug if either square goes out of bounds (> 63)
    pub fn new(from_sq: pos::Square, to_sq: pos::Square, type_of: MoveType) -> Move {
        debug_assert!(from_sq < 64, "from square is out of bounds!");
        debug_assert!(to_sq < 64, "to square is out of bounds!");

        Move {
            from_sq,
            to_sq,
            type_of,
            is_reversible: true,
        }
    }

    /// the square the moving piece is on, assuming the move hasnt been played yet
    #[inline(always)]
    pub fn from_sq(self) -> pos::Square {
        self.from_sq
    }

    /// the target square
    #[inline(always)]
    pub fn to_sq(self) -> pos::Square {
        self.to_sq
    }

    /// the type of the move
    #[inline(always)]
    pub fn type_of(self) -> MoveType {
        self.type_of
    }

    /// whether the move can be normally reversed or not, this is false for moves like captures,
    /// or ones that lose castling rights
    ///
    /// note that this is only set after a move has already been played, it is defaulted to `true`
    #[inline(always)]
    pub fn is_reversible(self) -> bool {
        self.is_reversible
    }

    /// converts the move to a string in the following format: `"e2e4"`   
    ///
    /// for promotions, no equal sign is used: `"e7e8q"`
    pub fn to_uci_fmt(self) -> String {
        let mut uci = format!("{}{}", pos::to_algn(self.from_sq), pos::to_algn(self.to_sq));

        match self.type_of {
            MoveType::Promotion(promoted) | MoveType::PromoCapture(promoted, _) => {
                uci.push(piece::as_char((promoted & !color::WHITE) | color::BLACK));
                ()
            }
            _ => (),
        }

        uci
    }

    /// converts a string in uci format to a move
    ///
    /// the format is the following: `"e2e4"`
    ///
    /// for promotions it doesn't matter if there is an equal sign or not, so both
    /// `"e7e8q"` and `"e7e8=q"` are valid
    ///
    /// the funtion assumes a string in correct format and doesn't do much error checking
    ///
    /// string correctness should be handled by a gui (really im just lazy)
    pub fn from_str_move(uci: &str, pos: &pos::Position) -> Self {
        let uci = uci.trim();
        assert!(uci.len() >= 4, "uci string '{uci}' is too short");
        assert!(uci.len() <= 6, "uci string '{uci}' is too long");

        let from = pos::string_to_sq(&uci.chars().take(2).collect());
        let to = pos::string_to_sq(&uci.chars().skip(2).take(2).collect());

        assert!(
            from < 64,
            "'{}' converts to an invalid index",
            uci.chars().take(2).collect::<String>()
        );
        assert!(
            to < 64,
            "'{}' converts to an invalid index",
            uci.chars().skip(2).take(2).collect::<String>()
        );

        let promo: char = uci.chars().nth_back(0).unwrap();
        let mut mov = Self::new(from, to, MoveType::Normal);
        let promo_piece;

        if pos.is_occupied(to) {
            mov.type_of = MoveType::Capture(pos.piece_on(to));
        }

        // if the last character of the move string is not a number,
        // then it indicates a promotion
        if promo.is_alphabetic() {
            promo_piece = (piece::from_char(promo.to_ascii_lowercase()) & !color::BLACK)
                | color::of(pos.piece_on(from));

            mov.type_of = match mov.type_of {
                MoveType::Capture(cap) => MoveType::PromoCapture(promo_piece, cap),
                _ => MoveType::Promotion(promo_piece),
            };

            return mov;
        }

        if pos.piece_on(from) & piece::PAWN != 0 {
            if from.abs_diff(to) == 16
            // two squares forward from either perspective
            {
                mov.type_of = MoveType::PawnTwoUp;
                return mov;
            } else if from.abs_diff(to) == 9 || from.abs_diff(to) == 7
            // diagonal
            {
                if let MoveType::Capture(_) = mov.type_of {
                } else {
                    mov.type_of = MoveType::EnPassant;
                    return mov;
                }
            }
        }

        if pos.piece_on(from) & piece::KING != 0 {
            if from as isize - to as isize == -2 {
                mov.type_of = MoveType::KingSideCastle;
                return mov;
            } else if from as isize - to as isize == 2 {
                mov.type_of = MoveType::QueenSideCastle;
                return mov;
            }
        }

        mov
    }
}

/// generates all pseudo legal pawn moves
pub fn pawn_moves(pos: &pos::Position, moves: &mut Vec<Move>, masks: &AttackMasks) {
    let side = pos.side_to_move();
    let mut pawns = pos.piece_bb(piece::PAWN | side);

    while pawns != bb::EMPTY {
        let from = pawns.serialize_once();
        let ep = match pos.ep_square() {
            Some(from) => 1 << from,
            None => 0,
        };

        let attacks = masks.pawn_attacks(side, from);
        let mut captures = attacks & pos.color_bb(color::other(side));
        let mut ep_captures = attacks & ep;

        let up1 = pos::ahead(from, side);

        let start_rank = match side {
            color::WHITE => pos::RANK_2,
            _ => pos::RANK_7,
        };

        let promo_rank = match side {
            color::WHITE => pos::RANK_8,
            _ => pos::RANK_1,
        };

        while captures != EMPTY {
            let to = captures.serialize_once();
            let cap = pos.piece_on(to);

            if pos::rank_of(to) == promo_rank {
                moves.push(Move::new(
                    from,
                    to,
                    MoveType::PromoCapture(piece::QUEEN | side, cap),
                ));
                moves.push(Move::new(
                    from,
                    to,
                    MoveType::PromoCapture(piece::ROOK | side, cap),
                ));
                moves.push(Move::new(
                    from,
                    to,
                    MoveType::PromoCapture(piece::BISHOP | side, cap),
                ));
                moves.push(Move::new(
                    from,
                    to,
                    MoveType::PromoCapture(piece::KNIGHT | side, cap),
                ));
            } else {
                moves.push(Move::new(from, to, MoveType::Capture(cap)));
            }
        }

        while ep_captures != EMPTY {
            moves.push(Move::new(
                from,
                ep_captures.serialize_once(),
                MoveType::EnPassant,
            ));
        }

        if !pos.is_occupied(up1) {
            if pos::rank_of(up1) == promo_rank {
                moves.push(Move::new(
                    from,
                    up1,
                    MoveType::Promotion(piece::QUEEN | side),
                ));
                moves.push(Move::new(
                    from,
                    up1,
                    MoveType::Promotion(piece::ROOK | side),
                ));
                moves.push(Move::new(
                    from,
                    up1,
                    MoveType::Promotion(piece::BISHOP | side),
                ));
                moves.push(Move::new(
                    from,
                    up1,
                    MoveType::Promotion(piece::KNIGHT | side),
                ));
            } else {
                moves.push(Move::new(from, up1, MoveType::Normal));
            }

            if pos::rank_of(from) == start_rank {
                let up2 = pos::ahead(up1, side);
                if !pos.is_occupied(up2) {
                    moves.push(Move::new(from, up2, MoveType::PawnTwoUp));
                }
            }
        }
    }
}

/// generates all pseudo legal knight moves
pub fn knight_moves(pos: &pos::Position, moves: &mut Vec<Move>, masks: &AttackMasks) {
    let side = pos.side_to_move();
    let mut knights = pos.piece_bb(piece::KNIGHT | side);

    while knights != bb::EMPTY {
        let from = knights.serialize_once();
        let attacks = masks.knight_attacks(from);
        let mut captures = attacks & pos.color_bb(color::other(side));
        let mut quiets = attacks & pos.empty_bb();

        while captures != bb::EMPTY {
            let to = captures.serialize_once();
            let cap = pos.piece_on(to);
            moves.push(Move::new(from, to, MoveType::Capture(cap)));
        }

        while quiets != bb::EMPTY {
            moves.push(Move::new(from, quiets.serialize_once(), MoveType::Normal));
        }
    }
}

/// generates all pseudo legal moves for rooks (and queens not moving diagonally)
pub fn rook_or_queen_moves(pos: &pos::Position, moves: &mut Vec<Move>, masks: &AttackMasks) {
    let side = pos.side_to_move();
    let mut rook_or_queens = pos.piece_bb(piece::ROOK | side) | pos.piece_bb(piece::QUEEN | side);

    while rook_or_queens != bb::EMPTY {
        let from = rook_or_queens.serialize_once();
        let attacks = masks.rook_attacks_rt(from, pos.occupied_bb());
        let mut captures = attacks & pos.color_bb(color::other(side));
        let mut quiets = attacks & pos.empty_bb();

        while captures != bb::EMPTY {
            let to = captures.serialize_once();
            let cap = pos.piece_on(to);
            moves.push(Move::new(from, to, MoveType::Capture(cap)));
        }

        while quiets != bb::EMPTY {
            moves.push(Move::new(from, quiets.serialize_once(), MoveType::Normal));
        }
    }
}

/// generates all pseudo legal moves for bishops (and queens moving only diagonally)
pub fn bishop_or_queen_moves(pos: &pos::Position, moves: &mut Vec<Move>, masks: &AttackMasks) {
    let side = pos.side_to_move();
    let mut bishop_or_queens =
        pos.piece_bb(piece::BISHOP | side) | pos.piece_bb(piece::QUEEN | side);

    while bishop_or_queens != bb::EMPTY {
        let from = bishop_or_queens.serialize_once();
        let attacks = masks.bishop_attacks_rt(from, pos.occupied_bb());
        let mut captures = attacks & pos.color_bb(color::other(side));
        let mut quiets = attacks & pos.empty_bb();

        while captures != bb::EMPTY {
            let to = captures.serialize_once();
            let cap = pos.piece_on(to);
            moves.push(Move::new(from, to, MoveType::Capture(cap)));
        }

        while quiets != bb::EMPTY {
            moves.push(Move::new(from, quiets.serialize_once(), MoveType::Normal));
        }
    }
}

/// generates all pseudo legal king moves
pub fn king_moves(pos: &pos::Position, moves: &mut Vec<Move>, masks: &AttackMasks) {
    let side = pos.side_to_move();
    let mut king = pos.piece_bb(piece::KING | side);

    let from = king.serialize_once();
    let attacks = masks.king_attacks(from);
    let mut captures = attacks & pos.color_bb(color::other(side));
    let mut quiets = attacks & pos.empty_bb();

    while captures != bb::EMPTY {
        let to = captures.serialize_once();
        let cap = pos.piece_on(to);
        moves.push(Move::new(from, to, MoveType::Capture(cap)));
    }

    while quiets != bb::EMPTY {
        moves.push(Move::new(from, quiets.serialize_once(), MoveType::Normal));
    }

    let (kcastle, qcastle) = match side {
        color::WHITE => (pos::WK_CASTLE, pos::WQ_CASTLE),
        _ => (pos::BK_CASTLE, pos::BQ_CASTLE),
    };

    if pos.castle_rights() & kcastle != 0
        && !pos.is_occupied(from + 1)
        && !pos.is_occupied(from + 2)
        && !pos.is_check(masks)
        && !bb::is_attacked(from + 1, &pos, color::other(side), masks)
        && !bb::is_attacked(from + 2, &pos, color::other(side), masks)
    {
        moves.push(Move::new(from, from + 2, MoveType::KingSideCastle));
    }

    if pos.castle_rights() & qcastle != 0
        && !pos.is_occupied(from - 1)
        && !pos.is_occupied(from - 2)
        && !pos.is_occupied(from - 3)
        && !pos.is_check(masks)
        && !bb::is_attacked(from - 1, &pos, color::other(side), masks)
        && !bb::is_attacked(from - 2, &pos, color::other(side), masks)
    {
        moves.push(Move::new(from, from - 2, MoveType::QueenSideCastle));
    }
}

/// generates all legal moves by first generating pseudo legal moves, and then filtering out the illegal ones
pub fn gen_legal(pos: &mut pos::Position, masks: &AttackMasks, zb: &ZobristValues) -> Vec<Move> {
    let mut moves = Vec::new();
    moves.reserve(238); // 238 is the max number of legal moves in any given position
    let side = pos.side_to_move();

    pawn_moves(pos, &mut moves, masks);
    knight_moves(pos, &mut moves, masks);
    rook_or_queen_moves(pos, &mut moves, masks);
    bishop_or_queen_moves(pos, &mut moves, masks);
    king_moves(pos, &mut moves, masks);

    fn is_legal(m: Move, pos: &mut pos::Position, masks: &AttackMasks, zb: &ZobristValues) -> bool {
        let cap = pos.fast_make(m, m.type_of() == MoveType::EnPassant, zb);
        // pos.make_move(m, zb);
        let is_legal = !pos.is_check(masks);
        pos.fast_unmake(m, cap, m.type_of() == MoveType::EnPassant, zb);
        // pos.unmake_move();

        is_legal
    }

    if pos.is_check(masks) {
        moves
            .into_iter()
            .filter(|&m| is_legal(m, pos, masks, zb))
            .collect()
    } else {
        moves
            .into_iter()
            .filter(|&m| {
                if pos.piece_on(m.from_sq()) & piece::KING != 0 {
                    !bb::is_attacked(m.to_sq(), &pos, color::other(side), masks)
                } else if bb::might_be_pinned(pos, m.from_sq()) {
                    is_legal(m, pos, masks, zb)
                } else {
                    true
                }
            })
            .collect()
    }
}
