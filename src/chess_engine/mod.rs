use std::ops::{Index, IndexMut};

use self::{gen_moves::Move, make_move::Unmove, zobrist::Zobrist};

pub mod book;
pub mod display;
pub mod fen;
pub mod gen_moves;
pub mod make_move;
pub mod search;
pub mod static_eval;
pub mod transposition_table;
pub mod zobrist;

#[rustfmt::skip]
const MAILBOX: [Option<u8>; 120] = [
    None, None,     None,     None,     None,     None,     None,     None,     None,     None,
    None, None,     None,     None,     None,     None,     None,     None,     None,     None,
    None, Some(0),  Some(1),  Some(2),  Some(3),  Some(4),  Some(5),  Some(6),  Some(7),  None,
    None, Some(8),  Some(9),  Some(10), Some(11), Some(12), Some(13), Some(14), Some(15), None,
    None, Some(16), Some(17), Some(18), Some(19), Some(20), Some(21), Some(22), Some(23), None,
    None, Some(24), Some(25), Some(26), Some(27), Some(28), Some(29), Some(30), Some(31), None,
    None, Some(32), Some(33), Some(34), Some(35), Some(36), Some(37), Some(38), Some(39), None,
    None, Some(40), Some(41), Some(42), Some(43), Some(44), Some(45), Some(46), Some(47), None,
    None, Some(48), Some(49), Some(50), Some(51), Some(52), Some(53), Some(54), Some(55), None,
    None, Some(56), Some(57), Some(58), Some(59), Some(60), Some(61), Some(62), Some(63), None,
    None, None,     None,     None,     None,     None,     None,     None,     None,     None,
    None, None,     None,     None,     None,     None,     None,     None,     None,     None,
];

#[rustfmt::skip]
const MAILBOX64: [u8; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28, 
    31, 32, 33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58,
    61, 62, 63, 64, 65, 66, 67, 68,
    71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88,
    91, 92, 93, 94, 95, 96, 97, 98,
];

pub const fn with_offset(from: u8, offset: i8) -> Option<u8> {
    MAILBOX[(MAILBOX64[from as usize] as i8 + offset) as usize]
}

const fn si(f: u8, r: u8) -> u8 {
    8 * r + f
}

const fn fr(si: u8) -> (u8, u8) {
    (si & 7, si >> 3)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

impl PieceColor {
    pub fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceType {
    Rook = 0,
    Knight = 1,
    Bishop = 2,
    Queen = 3,
    King = 4,
    Pawn = 5,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub c: PieceColor,
    pub t: PieceType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PieceColorArray<T>([T; 2]);
impl<T> Index<PieceColor> for PieceColorArray<T> {
    type Output = T;

    fn index(&self, index: PieceColor) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl<T> IndexMut<PieceColor> for PieceColorArray<T> {
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Clone)]
pub struct ChessState {
    pieces: [Option<Piece>; 64],
    turn: PieceColor,
    king_castle: PieceColorArray<bool>,
    queen_castle: PieceColorArray<bool>,
    en_passant_target: Option<u8>,

    halfmove_clock: u8,
    move_clock: u8,
    check: PieceColorArray<bool>,
    king_pos: PieceColorArray<u8>,
    hash: u64,

    unmove_stack: Vec<Unmove>,
}

impl Default for ChessState {
    fn default() -> Self {
        let mut ret = Self {
            pieces: [None; 64],
            king_pos: PieceColorArray([0, 0]),

            turn: PieceColor::White,
            king_castle: PieceColorArray([false, false]),
            queen_castle: PieceColorArray([false, false]),
            en_passant_target: None,
            check: PieceColorArray([false, false]),
            halfmove_clock: 0,
            move_clock: 1,
            hash: 0,

            unmove_stack: Vec::new(),
        };

        ret.hash = Zobrist::calc_hash(&ret);
        ret
    }
}

impl ChessState {
    pub fn is_square_attacked_by(&self, square: u8, attacker: PieceColor) -> bool {
        // Check if square is attacked orthogonally by a rook, queen or king
        for offset in PieceType::Rook.offsets() {
            let mut slid = false;
            let mut to = square;
            loop {
                to = match with_offset(to, *offset) {
                    Some(n) => n,
                    None => break,
                };

                match self.pieces[to as usize] {
                    Some(Piece {
                        c,
                        t: PieceType::Rook | PieceType::Queen,
                    }) if c == attacker => return true,
                    Some(Piece { c, t: PieceType::King }) if c == attacker && !slid => return true,
                    Some(_) => break,
                    _ => (),
                }

                slid = true;
            }
        }

        // Check if square is diagonally attacked by a bishop, queen or king
        for offset in PieceType::Bishop.offsets() {
            let mut slid = false;
            let mut to = square;
            loop {
                to = match with_offset(to, *offset) {
                    Some(n) => n,
                    None => break,
                };

                match self.pieces[to as usize] {
                    Some(Piece {
                        c,
                        t: PieceType::Bishop | PieceType::Queen,
                    }) if c == attacker => return true,
                    Some(Piece { c, t: PieceType::King }) if c == attacker && !slid => return true,
                    Some(_) => break,
                    _ => (),
                }

                slid = true;
            }
        }

        // Check if square is attacked by a knight
        for offset in PieceType::Knight.offsets() {
            let to = match with_offset(square, *offset) {
                Some(n) => n,
                None => continue,
            };

            match self.pieces[to as usize] {
                Some(Piece { c, t: PieceType::Knight }) if c == attacker => return true,
                _ => (),
            }
        }

        // Check if square is attacked by a pawn
        let backward = match attacker {
            PieceColor::White => -10,
            PieceColor::Black => 10,
        };

        for offset in [1 as i8, -1] {
            let to = match with_offset(square, backward + offset) {
                Some(n) => n,
                None => continue,
            };

            match self.pieces[to as usize] {
                Some(Piece { c, t: PieceType::Pawn }) if c == attacker => return true,
                _ => (),
            }
        }

        false
    }

    pub fn update_check(&mut self) {
        self.check[PieceColor::White] = self.is_square_attacked_by(self.king_pos[PieceColor::White], PieceColor::Black);
        self.check[PieceColor::Black] = self.is_square_attacked_by(self.king_pos[PieceColor::Black], PieceColor::White);
    }

    pub fn get_move(&mut self, m: &str) -> Option<Move> {
        self.gen_moves().iter().find(|mv| mv.to_string() == m).cloned()
    }
}
