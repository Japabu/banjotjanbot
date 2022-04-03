use super::{gen_moves::Move, ChessState, PieceColor, PieceColorArray, PieceType};

static mut ZOBRIST: Option<Zobrist> = None;
struct Zobrist {
    piece: PieceColorArray<[[u64; 64]; 6]>,
    turn: u64,
    queen_castle: PieceColorArray<u64>,
    king_castle: PieceColorArray<u64>,
    en_passant: [u64; 8],
}

pub fn init_zobrist() {
    let mut zobrist = Zobrist {
        piece: PieceColorArray([[[0; 64]; 6]; 2]),
        turn: 0,
        queen_castle: PieceColorArray([0; 2]),
        king_castle: PieceColorArray([0; 2]),
        en_passant: [0; 8],
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
        ZOBRIST = Some(zobrist);
    }
}

impl ChessState {
    pub fn hash_move(&mut self, m: &Move) {
        // let zobrist = unsafe { ZOBRIST.as_ref().unwrap() };
        // self.hash ^= zobrist.piece[self.turn][m.piece.t as usize][m.from];
        // self.hash ^= zobrist.piece[self.turn][m.piece.t as usize][m.to];
    }
}
