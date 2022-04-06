use std::ops::{Index, IndexMut};

use self::{make_move::Unmove, zobrist::Zobrist};

pub mod book;
pub mod display;
pub mod fen;
pub mod gen_moves;
pub mod make_move;
pub mod search;
pub mod static_eval;
pub mod transposition_table;
pub mod zobrist;

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
    check: bool,
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
            check: false,
            halfmove_clock: 0,
            move_clock: 1,
            hash: 0,

            unmove_stack: Vec::new(),
        };

        ret.hash = Zobrist::calc_hash(&ret);
        ret
    }
}
