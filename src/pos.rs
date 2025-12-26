use colored::Colorize;

use crate::{
    AttackMasks, ZobristValues, color,
    moves::{self, MoveType},
    piece::{
        self,
        bb::{self, BitboardUtil},
    },
    zobrist,
};

pub type Rank = isize;
pub type File = isize;
pub type Square = usize;

/// a position keeps track of each side's castling rights by encoding bits into a single u8 integer,
/// the first 2 bits are for white's castling rights, and the following 2 bits are for black's,
/// if the whole integer is 0, then neither side has any castling rights
pub type CastleRights = u8;

pub const NO_CASTLING: CastleRights = 0x0;
pub const WK_CASTLE: CastleRights = 0x2; // white, king side
pub const WQ_CASTLE: CastleRights = 0x4; // white, queen side
pub const BK_CASTLE: CastleRights = 0x8; // black, king side
pub const BQ_CASTLE: CastleRights = 0x10; // black, queen side

pub const FILE_A: File = 0;
pub const FILE_B: File = 1;
pub const FILE_C: File = 2;
pub const FILE_D: File = 3;
pub const FILE_E: File = 4;
pub const FILE_F: File = 5;
pub const FILE_G: File = 6;
pub const FILE_H: File = 7;

pub const RANK_1: Rank = 0;
pub const RANK_2: Rank = 1;
pub const RANK_3: Rank = 2;
pub const RANK_4: Rank = 3;
pub const RANK_5: Rank = 4;
pub const RANK_6: Rank = 5;
pub const RANK_7: Rank = 6;
pub const RANK_8: Rank = 7;

/// starting square of the white king's rook
pub const WK_ROOK_SQ: Square = 7;

/// starting square of the white queen's rook
pub const WQ_ROOK_SQ: Square = 0;

/// starting square of the black king's rook
pub const BK_ROOK_SQ: Square = 63;

/// starting square of the black queen's rook
pub const BQ_ROOK_SQ: Square = 56;

/// Fen string for the starting position
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// number of plies that `Position::rule50` has to be equal to for a position to be a draw
pub const RULE_50_PLIES: u8 = 100;

pub const QUEEN_VALUE: u32 = 9;
pub const ROOK_VALUE: u32 = 5;
pub const BISHOP_VALUE: u32 = 3;
pub const KNIGHT_VALUE: u32 = 3;
pub const PAWN_VALUE: u32 = 1;

/// struct containing the values of each piece type
/// uses integers as values are meant to be in `centipawns`
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PieceValues {
    pub pawn: u32,
    pub knight: u32,
    pub bishop: u32,
    pub rook: u32,
    pub queen: u32,
    pub king: u32,
}

/// struct containing all information about the state of a position, such as its **zobrist key**,
/// irreversible data, board representation, etc.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StateInfo {
    pub ep_square: Option<Square>,
    pub rule50: u8,
    pub castling: CastleRights,
    pub move_played: Option<moves::Move>,
    pub board: [piece::Piece; 64],
    pub piece_bb: [bb::Bitboard; 12],
    pub color_bb: [bb::Bitboard; 2],
    pub side: color::Color,
    pub ply: usize,
    pub key: zobrist::Key,
}

/// wrapper for the `StateInfo` struct,
/// additionally contains a vector of previous states for move unmaking purposes
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Position {
    st: StateInfo,
    history: Vec<StateInfo>,
}

/// gives both sides all castling rights
#[inline(always)]
pub fn set_all_true(rights: &mut CastleRights) {
    *rights |= WK_CASTLE | WQ_CASTLE | BK_CASTLE | BQ_CASTLE;
}

/// loses the right to castle king side for the side `color`
///
/// returns the zobrist value for the right lost, or 0 if the right was already lost previously
///
/// returns 0 and doesn't change the castling rights if `color` is neither `WHITE` nor `BLACK`
pub fn lose_kcastle_rights(
    rights: &mut CastleRights,
    color: color::Color,
    zb: &ZobristValues,
) -> u64 {
    let (right, zb_value) = match color {
        color::WHITE => (
            WK_CASTLE,
            if *rights & WK_CASTLE != 0 {
                // return the zobrist value for the right being lost, so we can update the position key with it
                zb.wk_castle
            } else {
                // if the castling right has already been lost, return 0 to avoid the position being hashed
                // as if the castling right was still present
                0
            },
        ),
        color::BLACK => (
            BK_CASTLE,
            if *rights & BK_CASTLE != 0 {
                zb.bk_castle
            } else {
                0
            },
        ),
        _ => (0, 0),
    };

    *rights &= !right;

    zb_value
}

/// loses the right to castle queen side for the side of `color`
///
/// returns the zobrist value for the right lost, or 0 if the right was already lost previously
///
/// returns 0 and doesn't change the castling rights if `color` is neither `WHITE` nor `BLACK`
pub fn lose_qcastle_rights(
    rights: &mut CastleRights,
    color: color::Color,
    zb: &ZobristValues,
) -> u64 {
    let (right, zb_value) = match color {
        color::WHITE => (
            WQ_CASTLE,
            if *rights & WQ_CASTLE != 0 {
                zb.wq_castle
            } else {
                0
            },
        ),
        color::BLACK => (
            BQ_CASTLE,
            if *rights & BQ_CASTLE != 0 {
                zb.bq_castle
            } else {
                0
            },
        ),
        _ => (0, 0),
    };

    *rights &= !right;

    zb_value
}

impl Position {
    pub fn blank() -> Self {
        Position {
            st: StateInfo {
                ep_square: None,
                rule50: 0,
                castling: NO_CASTLING,
                move_played: None,
                board: [piece::NONE; 64],
                piece_bb: [bb::EMPTY; 12],
                color_bb: [bb::EMPTY; 2],
                side: color::NONE,
                ply: 0,
                key: 0,
            },
            history: Vec::new(),
        }
    }

    /// creates a `Position` object from a `StateInfo` object at an index of the history (including the current state)
    ///
    /// the returned object has a clear history
    pub fn from_ply(&self, ply: usize) -> Self {
        Position {
            st: self.history_i()[ply],
            history: Vec::new(),
        }
    }

    /// the color of the current side to move
    #[inline(always)]
    pub fn side_to_move(&self) -> color::Color {
        self.st.side
    }

    /// a vector containing past states of the position
    #[inline(always)]
    pub fn history(&self) -> &Vec<StateInfo> {
        &self.history
    }

    /// a vector containing past states of the position including the current
    ///
    /// (possibly slow as it clones both the history and the current state)
    #[inline(always)]
    pub fn history_i(&self) -> Vec<StateInfo> {
        let mut h = self.history.clone();
        h.append(&mut vec![self.st]);
        h
    }

    /// the square where en passant is possible, if any
    #[inline(always)]
    pub fn ep_square(&self) -> Option<Square> {
        self.st.ep_square
    }

    /// a counter for the 50 move rule, increases per *ply*,
    /// so a position would be a draw if it reaches 100
    #[inline(always)]
    pub fn rule50(&self) -> u8 {
        self.st.rule50
    }

    /// an integer containing the castle rights for both players
    #[inline(always)]
    pub fn castle_rights(&self) -> CastleRights {
        self.st.castling
    }

    /// the zobrist key for the current position
    #[inline(always)]
    pub fn key(&self) -> u64 {
        self.st.key
    }

    /// an 8x8 board represented as an array with 64 indices, if an index contains no piece,
    /// its value is 0 `(piece::NONE)`
    #[inline(always)]
    pub fn board(&self) -> &[piece::Piece; 64] {
        &self.st.board
    }

    /// an array containing bitboards for all piece types
    #[inline(always)]
    pub fn piece_bb(&self, piece: piece::Piece) -> bb::Bitboard {
        self.st.piece_bb[bb::p_to_idx(piece)]
    }

    /// an array containing bitboards for all pieces of a given color
    #[inline(always)]
    pub fn color_bb(&self, color: color::Color) -> bb::Bitboard {
        self.st.color_bb[bb::c_to_idx(color)]
    }

    /// a bitboard of all occupied squares
    #[inline(always)]
    pub fn occupied_bb(&self) -> bb::Bitboard {
        self.color_bb(color::WHITE) | self.color_bb(color::BLACK)
    }

    /// a bitboard of all empty squares
    #[inline(always)]
    pub fn empty_bb(&self) -> bb::Bitboard {
        !self.occupied_bb()
    }

    /// get the piece on a given square
    #[inline(always)]
    pub fn piece_on(&self, square: Square) -> piece::Piece {
        self.st.board[square]
    }

    /// get the piece on a given file and rank
    #[inline(always)]
    pub fn piece_on_fr(&self, file: File, rank: Rank) -> piece::Piece {
        self.st.board[make_sq(file, rank)]
    }

    /// the current ply of the position
    #[inline(always)]
    pub fn ply(&self) -> usize {
        self.st.ply
    }

    /// returns true if a square is occupied by any piece, false otherwise
    #[inline(always)]
    pub fn is_occupied(&self, square: Square) -> bool {
        self.st.board[square] != piece::NONE
    }

    /// the last move played in the position, if any
    #[inline(always)]
    pub fn move_played(&self) -> Option<moves::Move> {
        self.st.move_played
    }

    /// returns the list of all moves leading up to the current position
    #[inline(always)]
    pub fn moves(&self) -> Vec<moves::Move> {
        let mut ms = self
            .history
            .iter()
            .filter_map(|st| st.move_played)
            .collect::<Vec<moves::Move>>();

        if let Some(mov) = self.st.move_played {
            ms.append(&mut vec![mov]);
        }

        ms
    }

    /// returns the list of all moves leading up to the current position wrapped in an `Option<>`
    ///
    /// includes the `None` move of a starting position
    pub fn moves_opt(&self) -> Vec<Option<moves::Move>> {
        let mut ms = self
            .history
            .iter()
            .map(|st| st.move_played)
            .collect::<Vec<Option<moves::Move>>>();

        ms.append(&mut vec![self.st.move_played]);

        ms
    }

    /// returns true if the king of the side to move is in check
    #[inline(always)]
    pub fn is_check(&self, masks: &AttackMasks) -> bool {
        bb::is_attacked(
            self.piece_bb(piece::KING | self.st.side).serialize_once(),
            self,
            color::other(self.st.side),
            masks,
        )
    }

    /// returns the amount of material a side has using the standard values for pieces
    #[inline(always)]
    pub fn count_material(&self, side: color::Color) -> i32 {
        (self.piece_bb(piece::QUEEN | side).count_ones() * QUEEN_VALUE
            + self.piece_bb(piece::ROOK | side).count_ones() * ROOK_VALUE
            + self.piece_bb(piece::BISHOP | side).count_ones() * BISHOP_VALUE
            + self.piece_bb(piece::KNIGHT | side).count_ones() * KNIGHT_VALUE
            + self.piece_bb(piece::PAWN | side).count_ones() * PAWN_VALUE) as i32
    }

    /// returns the amount of material a side has using custom values for pieces
    #[inline(always)]
    pub fn count_material_custom(&self, side: color::Color, values: PieceValues) -> i32 {
        (self.piece_bb(piece::QUEEN | side).count_ones() * values.queen
            + self.piece_bb(piece::ROOK | side).count_ones() * values.rook
            + self.piece_bb(piece::BISHOP | side).count_ones() * values.bishop
            + self.piece_bb(piece::KNIGHT | side).count_ones() * values.knight
            + self.piece_bb(piece::PAWN | side).count_ones() * values.pawn) as i32
    }

    /// returns the material difference between white (+) and black (-)
    #[inline(always)]
    pub fn material_diff(&self) -> i32 {
        self.count_material(color::WHITE) - self.count_material(color::BLACK)
    }

    /// returns true if `side` has insufficient material to force checkmate
    ///
    #[inline(always)]
    pub fn insufficient_material(&self, side: color::Color) -> bool {
        let queens = self.piece_bb(piece::QUEEN | side).count_ones();
        let rooks = self.piece_bb(piece::ROOK | side).count_ones();
        let pawns = self.piece_bb(piece::PAWN | side).count_ones();
        let bishops = self.piece_bb(piece::BISHOP | side).count_ones();
        let knights = self.piece_bb(piece::KNIGHT | side).count_ones();

        queens == 0
            && rooks == 0
            && pawns == 0
            && ((bishops == 0 && knights < 3) || (bishops == 1 && knights == 0))
    }
}

impl Position {
    /// takes a FEN string and creates a `Position` object with it
    ///
    /// the function assumes that the string is in correct format,
    /// otherwise, it may give funky results
    pub fn from_fen(fen_str: &str, zb: &ZobristValues) -> Self {
        let mut pos = Self::blank();

        let mut file = FILE_A;
        let mut rank = RANK_8;
        let mut str_idx = 0;

        for ch in fen_str.chars().take(fen_str.find(' ').unwrap()) {
            if ch == '/' {
                file = FILE_A;
                rank -= 1;
            } else if ch.is_numeric() {
                file += ch as isize - b'0' as isize;
            } else {
                pos.put_piece(piece::from_char(ch), make_sq(file, rank), zb);
                file += 1;
            }

            str_idx += 1;
        }

        str_idx += 1;

        pos.st.side = if fen_str.chars().nth(str_idx).unwrap() == 'w' {
            color::WHITE
        } else {
            color::BLACK
        };

        str_idx += 2;

        for ch in fen_str.chars().skip(str_idx).take_while(|ch| *ch != ' ') {
            match ch {
                '-' => {
                    pos.st.castling = NO_CASTLING;
                }
                'K' => {
                    pos.st.castling |= WK_CASTLE;
                }
                'Q' => {
                    pos.st.castling |= WQ_CASTLE;
                }
                'k' => {
                    pos.st.castling |= BK_CASTLE;
                }
                'q' => {
                    pos.st.castling |= BQ_CASTLE;
                }
                _ => panic!("I don't think this is a fen string"),
            }

            str_idx += 1;
        }

        str_idx += 1;

        if fen_str.chars().nth(str_idx).unwrap() == '-' {
            pos.st.ep_square = None;
            str_idx += 2;
        } else {
            pos.st.ep_square = Some(string_to_sq(
                &fen_str.chars().skip(str_idx).take(2).collect(),
            ));
            str_idx += 3;
        }

        let tmp: String = fen_str
            .chars()
            .skip(str_idx)
            .take_while(|&ch| ch != ' ')
            .collect();
        pos.st.rule50 = tmp.parse().unwrap();

        pos.st.key = zobrist::hash(&pos, zb);

        pos.history.reserve(400); // 400 is compltely arbitrary

        pos
    }

    /// prints a visual representation of the board
    pub fn visualize(&self) {
        println!("");
        println!("+---+---+---+---+---+---+---+---+");
        for rank in (RANK_1..=RANK_8).rev() {
            for file in FILE_A..=FILE_H {
                print!("| {} ", piece::as_char(self.piece_on_fr(file, rank)));
            }
            println!("| {}", rank + 1);
            println!("+---+---+---+---+---+---+---+---+");
        }
        println!("  a   b   c   d   e   f   g   h\n");
    }

    /// prints a visual representation of the board, with the square indices instead of pieces
    ///
    /// the formatting is wonky, only used for debugging purposes
    pub fn visualize_indices(&self) {
        println!("+---+---+---+---+---+---+---+---+");
        for rank in (RANK_1..=RANK_8).rev() {
            for file in FILE_A..=FILE_H {
                print!("| {} ", make_sq(file, rank));
            }
            println!("| {}", rank + 1);
            println!("+---+---+---+---+---+---+---+---+");
        }
        println!("  a   b   c   d   e   f   g   h\n");
    }

    /// prints a visual representation of the board, but smaller than that of `Position::visualize()`
    pub fn visualize_smaller(&self) {
        println!("");
        for rank in (RANK_1..=RANK_8).rev() {
            print!(" ");
            for file in FILE_A..=FILE_H {
                print!(
                    "{} ",
                    if piece::as_char(self.piece_on_fr(file, rank)) == ' ' {
                        '.'
                    } else {
                        piece::as_char(self.piece_on_fr(file, rank))
                    }
                );
            }
            println!("   {}", rank + 1);
        }
        println!("\n a b c d e f g h\n");
    }

    /// prints a pretty visual representation of the board, using UTF-8 symbols for the chess pieces,
    /// and a colored board
    pub fn visualize_pretty(&self) {
        for rank in (RANK_1..=RANK_8).rev() {
            for file in FILE_A..=FILE_H {
                if (rank + file) % 2 == 0 {
                    print!(
                        "{}{}",
                        piece::as_symbol(self.piece_on_fr(file, rank))
                            .black()
                            .on_custom_color((184, 135, 98)),
                        " ".on_custom_color((184, 135, 98))
                    );
                } else {
                    print!(
                        "{}{}",
                        piece::as_symbol(self.piece_on_fr(file, rank))
                            .black()
                            .on_custom_color((237, 214, 176)),
                        " ".on_custom_color((237, 214, 176))
                    );
                }
            }
            println!(" {}", rank + 1);
        }
        println!(" a b c d e f g h\n");
    }

    /// **plays a move**
    ///
    /// the function doesn't check for move legality, and assumes that the type of the move is correct,
    /// for example it would technically allow you to play a pawn push disguised as a king side castle
    pub fn make_move(&mut self, mov: moves::Move, zb: &ZobristValues) {
        self.history.push(self.st);
        self.st.rule50 += 1;
        self.st.move_played = Some(mov);

        self.st.ply += 1;

        if let Some(square) = self.st.ep_square {
            self.st.ep_square = None;
            self.st.key ^= zb.ep_files[file_of(square) as usize];
        }

        let moving_piece = self.st.board[mov.from_sq()];

        if moving_piece & piece::PAWN != 0 {
            self.st.rule50 = 0;
            self.st.move_played.unwrap().is_reversible = false;
        }

        if moving_piece & piece::KING != 0 {
            self.st.key ^= lose_kcastle_rights(&mut self.st.castling, self.st.side, zb);
            self.st.key ^= lose_qcastle_rights(&mut self.st.castling, self.st.side, zb);
        } else if moving_piece & piece::ROOK != 0 {
            self.st.key ^= match file_of(mov.from_sq()) {
                FILE_A => lose_qcastle_rights(&mut self.st.castling, self.st.side, zb),
                FILE_H => lose_kcastle_rights(&mut self.st.castling, self.st.side, zb),
                _ => 0,
            }
        }

        let rook_captured = match mov.type_of() {
            MoveType::Capture(cap) | MoveType::PromoCapture(_, cap) => cap & piece::ROOK != 0,
            _ => false,
        };

        if rook_captured {
            self.st.key ^= match self.st.side {
                color::WHITE => match mov.to_sq() {
                    BQ_ROOK_SQ => lose_qcastle_rights(&mut self.st.castling, color::BLACK, zb),
                    BK_ROOK_SQ => lose_kcastle_rights(&mut self.st.castling, color::BLACK, zb),
                    _ => 0,
                },
                color::BLACK => match mov.to_sq() {
                    WQ_ROOK_SQ => lose_qcastle_rights(&mut self.st.castling, color::WHITE, zb),
                    WK_ROOK_SQ => lose_kcastle_rights(&mut self.st.castling, color::WHITE, zb),
                    _ => 0,
                },
                _ => 0,
            }
        }

        match mov.type_of() {
            MoveType::Normal => {
                self.move_piece(mov.from_sq(), mov.to_sq(), zb);
            }
            MoveType::Capture(_) => {
                self.move_piece(mov.from_sq(), mov.to_sq(), zb);

                self.st.rule50 = 0;
                self.st.move_played.unwrap().is_reversible = false;
            }
            MoveType::PawnTwoUp => {
                self.move_piece(mov.from_sq(), mov.to_sq(), zb);

                let sq_behind = behind(mov.to_sq(), self.st.side);

                self.st.ep_square = Some(sq_behind);
                self.st.key ^= zb.ep_files[file_of(sq_behind) as usize];
            }
            MoveType::Promotion(promoted) | MoveType::PromoCapture(promoted, _) => {
                self.put_piece(promoted, mov.to_sq(), zb);
                self.remove_piece(mov.from_sq(), zb);

                self.st.move_played.unwrap().is_reversible = false;
            }
            MoveType::EnPassant => {
                self.move_piece(mov.from_sq(), mov.to_sq(), zb);
                self.remove_piece(behind(mov.to_sq(), self.st.side), zb);

                self.st.move_played.unwrap().is_reversible = false;
            }
            MoveType::KingSideCastle => {
                self.move_piece(mov.from_sq(), mov.to_sq(), zb);

                let (rook_from, rook_to): (Square, Square) = if self.st.side == color::WHITE {
                    (WK_ROOK_SQ, WK_ROOK_SQ - 2)
                } else {
                    (BK_ROOK_SQ, BK_ROOK_SQ - 2)
                };

                self.move_piece(rook_from, rook_to, zb);
                self.st.move_played.unwrap().is_reversible = false;
            }
            MoveType::QueenSideCastle => {
                self.move_piece(mov.from_sq(), mov.to_sq(), zb);

                let (rook_from, rook_to): (Square, Square) = if self.st.side == color::WHITE {
                    (WQ_ROOK_SQ, WQ_ROOK_SQ + 3)
                } else {
                    (BQ_ROOK_SQ, BQ_ROOK_SQ + 3)
                };

                self.move_piece(rook_from, rook_to, zb);
                self.st.move_played.unwrap().is_reversible = false;
            }
        }

        color::switch(&mut self.st.side);
        self.st.key ^= zb.black_to_move;
    }

    /// unmakes the last move played in a position
    /// by simply setting the current state to the last one in the `history` vector
    ///
    /// **panics** in debug if there are no moves to be unmade
    pub fn unmake_move(&mut self) {
        debug_assert!(
            self.history.len() != 0,
            "tried to unmake move on a start position"
        );

        self.st = *self.history.last().unwrap();
        self.history.pop();
    }

    /// returns true if a position has occured at least 3 times, otherwise false
    pub fn is_3_rep(&self) -> bool {
        if let Some(mov) = self.st.move_played {
            if !mov.is_reversible() {
                return false;
            }
        }

        // 6 is the minimum number of plies needed for a 3-fold repetition to be possible
        if self.history.len() < 6 {
            return false;
        }

        let mut idx = self.st.ply - 4;
        let mut cnt = 0;

        loop {
            if self.history[idx].key == self.st.key {
                cnt += 1;
                if cnt == 2 {
                    return true;
                }
            }

            if let Some(mov) = self.history[idx].move_played {
                if !mov.is_reversible() {
                    break;
                }
            }

            if idx < 2 {
                break;
            }

            idx -= 2;
        }

        false
    }
}

impl Position {
    fn put_piece(&mut self, piece: piece::Piece, square: Square, zb: &ZobristValues) {
        if self.is_occupied(square) {
            self.remove_piece(square, zb);
        }

        self.st.board[square] = piece;

        self.piece_bb_mut(piece).set_bit(square);
        self.color_bb_mut(piece).set_bit(square);

        self.st.key ^= zb.piece_sq[bb::p_to_idx(piece)][square];
    }

    fn remove_piece(&mut self, square: Square, zb: &ZobristValues) {
        self.st.key ^= zb.piece_sq[bb::p_to_idx(self.st.board[square])][square];

        self.piece_bb_mut(self.st.board[square]).pop_bit(square);
        self.color_bb_mut(self.st.board[square]).pop_bit(square);

        self.st.board[square] = piece::NONE;
    }

    fn move_piece(&mut self, from: Square, to: Square, zb: &ZobristValues) {
        self.put_piece(self.st.board[from], to, zb);
        self.remove_piece(from, zb);
    }

    /// moves a piece, but without making any incremental updates,
    ///
    /// returns the captured piece if any, for (fast) unmaking purposes
    ///
    /// used in legal move generation as a faster alternative to `Position::make_move()`,
    /// since we are going to end up unmaking all the moves right after making them, we don't
    /// need to make any incremental updates
    ///
    /// note: turns out it's about the same speed as regular make/unmake
    /// but using that breaks the movegen somehow?
    pub(crate) fn fast_make(
        &mut self,
        mov: moves::Move,
        ep: bool,
        zb: &ZobristValues,
    ) -> piece::Piece {
        let cap = self.st.board[match ep {
            true => behind(mov.to_sq(), self.st.side),
            false => mov.to_sq(),
        }];

        self.move_piece(mov.from_sq(), mov.to_sq(), zb);
        if ep {
            self.remove_piece(behind(mov.to_sq(), self.st.side), zb);
        }

        cap
    }

    /// undoes a move made with `Position::fast_make()`
    pub(crate) fn fast_unmake(
        &mut self,
        mov: moves::Move,
        cap: piece::Piece,
        ep: bool,
        zb: &ZobristValues,
    ) {
        self.move_piece(mov.to_sq(), mov.from_sq(), zb);

        if cap != piece::NONE {
            if ep {
                self.put_piece(cap, behind(mov.to_sq(), self.st.side), zb);
            } else {
                self.put_piece(cap, mov.to_sq(), zb);
            }
        }
    }

    fn piece_bb_mut(&mut self, piece: piece::Piece) -> &mut bb::Bitboard {
        &mut self.st.piece_bb[bb::p_to_idx(piece)]
    }

    fn color_bb_mut(&mut self, color: color::Color) -> &mut bb::Bitboard {
        &mut self.st.color_bb[bb::c_to_idx(color::of(color))]
    }
}

/// takes a file and rank number and returns the equivalent square index
pub fn make_sq(file: File, rank: Rank) -> Square {
    debug_assert!(file <= FILE_H, "file index is out of bounds");
    debug_assert!(rank <= RANK_8, "rank index is out of bounds");

    ((rank << 3) + file) as Square
}

/// takes a square index and returns the equivalent file and rank numbers
pub fn make_tuple(square: Square) -> (File, Rank) {
    debug_assert!(square < 64, "square index is out of bounds");

    (square as File & 7, square as Rank >> 3)
}

/// takes a square index and returns the equivalent file
pub fn file_of(square: Square) -> File {
    debug_assert!(square < 64, "square index is out of bounds");

    square as File & 7
}

/// takes a square index and returns the equivalent rank
pub fn rank_of(square: Square) -> Rank {
    debug_assert!(square < 64, "square index is out of bounds");

    square as Rank >> 3
}

/// converts a `String` in algebraic notation to a square index
pub fn string_to_sq(string: &String) -> Square {
    str_to_sq(&string)
}

/// converts a string literal in algebraic notation to a square index
pub fn str_to_sq(string: &str) -> Square {
    debug_assert_eq!(
        string.len(),
        2,
        "input string '{}' has to be 2 characters",
        string
    );

    let file = (string.chars().nth(0).unwrap() as u8 - b'a') as File;
    let rank = (string.chars().nth(1).unwrap() as u8 - b'1') as Rank;

    make_sq(file, rank)
}

/// converts a square index to its equivalent algebraic notation
pub fn to_algn(square: Square) -> String {
    debug_assert!(square < 64, "invalid square index");

    let (file, rank) = make_tuple(square);

    let mut string = String::new();
    string.push((file as u8 + b'a') as char);
    string.push((rank as u8 + b'1') as char);

    string
}

/// returns the square behind `square` from the perspective of the side to move
pub fn behind(square: Square, color: color::Color) -> Square {
    match color {
        color::WHITE => square - 8,
        _ => square + 8,
    }
}

/// returns the square ahead of `square` from the perspective of `color`
pub fn ahead(square: Square, color: color::Color) -> Square {
    match color {
        color::WHITE => square + 8,
        _ => square - 8,
    }
}

// demon go get a job
