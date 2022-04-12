use rand::Rng;

use super::{gen_moves::Move, ChessState, Piece, PieceColor, PieceColorArray, PieceType};

#[rustfmt::skip]
const PAWN_VALUES_MIDDLEGAME: [i32; 64] = [
     0,   0,   0,   0,   0,   0,  0,   0,
    98, 134,  61,  95,  68, 126, 34, -11,
    -6,   7,  26,  31,  65,  56, 25, -20,
   -14,  13,   6,  21,  23,  12, 17, -23,
   -27,  -2,  -5,  12,  17,   6, 10, -25,
   -26,  -4,  -4, -10,   3,   3, 33, -12,
   -35,  -1, -20, -23, -15,  24, 38, -22,
     0,   0,   0,   0,   0,   0,  0,   0,
];

#[rustfmt::skip]
const PAWN_VALUES_ENDGAME: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const KNIGHT_VALUES_MIDDLEGAME: [i32; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
const KNIGHT_VALUES_ENDGAME: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
const BISHOP_VALUES_MIDDLEGAME: [i32; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
const BISHOP_VALUES_ENDGAME: [i32; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
const ROOK_VALUES_MIDDLEGAME: [i32; 64] = [
    32,  42,  32,  51, 63,  9,  31,  43,
    27,  32,  58,  62, 80, 67,  26,  44,
    -5,  19,  26,  36, 17, 45,  61,  16,
   -24, -11,   7,  26, 24, 35,  -8, -20,
   -36, -26, -12,  -1,  9, -7,   6, -23,
   -45, -25, -16, -17,  3,  0,  -5, -33,
   -44, -16, -20,  -9, -1, 11,  -6, -71,
   -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
const ROOK_VALUES_ENDGAME: [i32; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
const QUEEN_VALUES_MIDDLEGAME: [i32; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
const QUEEN_VALUES_ENDGAME: [i32; 64] = [
     -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
const KING_VALUES_MIDDLEGAME: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

#[rustfmt::skip]
const KING_VALUES_ENDGAME: [i32; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43,
];

const LVA_MVV: [[i32; 6]; 6] = [
    [420, 220, 320, 520, 620, 120], // r captures r,n,b,q,k,p
    [440, 240, 340, 540, 640, 140], // n captures r,n,b,q,k,p
    [430, 230, 330, 530, 630, 130], // b captures r,n,b,q,k,p
    [410, 210, 310, 510, 610, 110], // q captures r,n,b,q,k,p
    [400, 200, 300, 500, 600, 100], // k captures r,n,b,q,k,p
    [450, 250, 350, 550, 650, 150], // p captures r,n,b,q,k,p
];

impl PieceType {
    const fn mat_value<const IS_ENDGAME: bool>(&self) -> i32 {
        if IS_ENDGAME {
            match self {
                PieceType::Pawn => 94,
                PieceType::Knight => 281,
                PieceType::Bishop => 297,
                PieceType::Rook => 512,
                PieceType::Queen => 936,
                PieceType::King => 0,
            }
        } else {
            match self {
                PieceType::Pawn => 82,
                PieceType::Knight => 337,
                PieceType::Bishop => 365,
                PieceType::Rook => 477,
                PieceType::Queen => 1025,
                PieceType::King => 0,
            }
        }
    }

    const fn middlegame_weight(&self) -> i32 {
        match self {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 1,
            PieceType::Rook => 2,
            PieceType::Queen => 4,
            PieceType::King => 0,
        }
    }
}

impl Piece {
    pub const fn pos_value<const IS_ENDGAME: bool>(&self, mut square: usize) -> i32 {
        square = match self.c {
            PieceColor::White => square ^ 56,
            PieceColor::Black => square,
        };

        if IS_ENDGAME {
            match self.t {
                PieceType::Pawn => PAWN_VALUES_ENDGAME[square],
                PieceType::Knight => KNIGHT_VALUES_ENDGAME[square],
                PieceType::Bishop => BISHOP_VALUES_ENDGAME[square],
                PieceType::Rook => ROOK_VALUES_ENDGAME[square],
                PieceType::Queen => QUEEN_VALUES_ENDGAME[square],
                PieceType::King => KING_VALUES_ENDGAME[square],
            }
        } else {
            match self.t {
                PieceType::Pawn => PAWN_VALUES_MIDDLEGAME[square],
                PieceType::Knight => KNIGHT_VALUES_MIDDLEGAME[square],
                PieceType::Bishop => BISHOP_VALUES_MIDDLEGAME[square],
                PieceType::Rook => ROOK_VALUES_MIDDLEGAME[square],
                PieceType::Queen => QUEEN_VALUES_MIDDLEGAME[square],
                PieceType::King => KING_VALUES_MIDDLEGAME[square],
            }
        }
    }
}

impl ChessState {
    pub fn static_eval(&self) -> i32 {
        let mut middlegame_values = PieceColorArray([0, 0]);
        let mut endgame_values = PieceColorArray([0, 0]);
        let mut is_middlegame = 0;
        for square in 0..64 {
            if let Some(piece) = self.pieces[square] {
                middlegame_values[piece.c] += piece.t.mat_value::<false>() + piece.pos_value::<false>(square);
                endgame_values[piece.c] += piece.t.mat_value::<true>() + piece.pos_value::<true>(square);
                is_middlegame += piece.t.middlegame_weight();
            }
        }

        let middlegame_score = middlegame_values[self.turn] - middlegame_values[self.turn.opposite()];
        let endgame_score = endgame_values[self.turn] - endgame_values[self.turn.opposite()];
        is_middlegame = is_middlegame.min(24);
        let is_endgame = 24 - is_middlegame;
        (middlegame_score * is_middlegame + endgame_score * is_endgame) / 24
    }
}

impl Move {
    pub fn static_eval(&self) -> i32 {
        let mut v = rand::thread_rng().gen_range(-10..=10);

        if let Some(t) = self.capture {
            v += LVA_MVV[self.piece_type as usize][t as usize];
        }

        if let Some(t) = self.promote_to {
            v += t.mat_value::<false>();
        }

        // TODO: Check if square is attacked by pawn

        v
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{static_eval::LVA_MVV, ChessState, PieceType};

    #[test]
    fn king_activity_endgame_test() {
        let state = ChessState::from_fen("7k/4p3/8/8/8/4P3/8/K7 w - - 0 1").unwrap();
        let eval_0 = state.static_eval();

        let state = ChessState::from_fen("7k/4p3/8/8/8/4P3/1K6/8 w - - 0 1").unwrap();
        let eval_1 = state.static_eval();

        assert!(eval_1 > eval_0);
    }

    #[test]
    fn mvv_lva_test() {
        for attacker in [
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Queen,
            PieceType::Pawn,
        ] {
            for victim in [PieceType::Rook, PieceType::Bishop, PieceType::Knight, PieceType::Queen, PieceType::Pawn] {
                let (higher_value, lower_value) = if attacker.mat_value::<false>() > victim.mat_value::<false>() {
                    (attacker, victim)
                } else {
                    (victim, attacker)
                };

                assert!(
                    LVA_MVV[lower_value as usize][higher_value as usize] >= LVA_MVV[higher_value as usize][lower_value as usize],
                    "{:?} captures {:?}",
                    attacker,
                    victim
                );
            }
        }
    }
}
