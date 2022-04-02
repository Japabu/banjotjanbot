use std::ops::{Index, IndexMut};

pub mod display;
pub mod search;
pub mod fen;
pub mod gen_moves;
pub mod make_move;
pub mod static_eval;

const fn si(f: u8, r: u8) -> usize {
    return (8 * r + f) as usize;
}

const fn fr(si: usize) -> (u8, u8) {
    return ((si & 7) as u8, (si >> 3) as u8);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub fn oppo(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Piece {
    pub c: PieceColor,
    pub t: PieceType,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ChessState {
    pieces: [Option<Piece>; 64],
    turn: PieceColor,
    king_castle: PieceColorArray<bool>,
    queen_castle: PieceColorArray<bool>,
    en_passant_target: Option<usize>,
    
    halfmove_count: u32,
    move_count: u32,
    check: bool,
    king_pos: PieceColorArray<usize>,
}

impl Default for ChessState {
    fn default() -> Self {
        Self {
            pieces: [None; 64],
            king_pos: PieceColorArray([0, 0]),

            turn: PieceColor::White,
            king_castle: PieceColorArray([false, false]),
            queen_castle: PieceColorArray([false, false]),
            en_passant_target: None,
            check: false,
            halfmove_count: 0,
            move_count: 1,
        }
    }
}
