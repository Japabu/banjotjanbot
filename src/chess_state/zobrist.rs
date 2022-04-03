use std::sync::RwLock;

use super::{gen_moves::Move, ChessState, PieceColor, PieceColorArray, PieceType};

static mut ZOBRIST: Option<RwLock<Zobrist>> = None;
pub struct Zobrist {
    piece: PieceColorArray<[[u64; 64]; 6]>,
    black_to_move: u64,
    queen_castle: PieceColorArray<u64>,
    king_castle: PieceColorArray<u64>,
    en_passant: [u64; 8],
}

impl Zobrist {
    pub fn init() {
        let mut zobrist = Zobrist {
            piece: PieceColorArray([[[0; 64]; 6]; 2]),
            black_to_move: rand::random(),
            queen_castle: PieceColorArray(rand::random()),
            king_castle: PieceColorArray(rand::random()),
            en_passant: rand::random(),
        };

        for color in [PieceColor::White, PieceColor::Black] {
            for pt in [
                PieceType::Pawn,
                PieceType::Knight,
                PieceType::Bishop,
                PieceType::Rook,
                PieceType::Queen,
                PieceType::King,
            ] {
                for sq in 0..64 {
                    zobrist.piece[color][pt as usize][sq] = rand::random();
                }
            }
        }

        for file in 0..8 {
            zobrist.en_passant[file] = rand::random();
        }

        unsafe {
            ZOBRIST = Some(RwLock::new(zobrist));
        }
    }
}

impl ChessState {
    pub fn calc_hash(&mut self) {
        let zobrist = unsafe { ZOBRIST.as_ref() }.unwrap().read().unwrap();

        self.hash = 0;

        if self.turn == PieceColor::Black {
            self.hash ^= zobrist.black_to_move;
        }

        for (sq, piece) in self.pieces.iter().enumerate() {
            if let Some(p) = piece {
                self.hash ^= zobrist.piece[p.c][p.t as usize][sq];
            }
        }

        for color in [PieceColor::White, PieceColor::Black] {
            if self.queen_castle[color] {
                self.hash ^= zobrist.queen_castle[color];
            }

            if self.king_castle[color] {
                self.hash ^= zobrist.king_castle[color];
            }
        }

        if let Some(sq) = self.en_passant_target {
            self.hash ^= zobrist.en_passant[sq % 8];
        }
    }

    pub fn inc_update(&mut self, m: &Move) {
        let zobrist = unsafe { ZOBRIST.as_ref() }.unwrap().read().unwrap();

        self.hash ^= zobrist.piece[self.turn][m.pt as usize][m.from];

        if let Some(p) = m.capture {
            self.hash ^= zobrist.piece[self.turn][p as usize][m.to];
        }

        self.hash ^= zobrist.piece[self.turn][m.pt as usize][m.to];
    }
}
