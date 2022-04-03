use super::{gen_moves::Move, ChessState, Piece, PieceColor, PieceType};

const PAWN_VALUES: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_VALUES: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_VALUES: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_VALUES: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

const QUEEN_VALUES: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const KING_VALUES: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

const KING_ENDGAME_VALUES: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];

impl Piece {
    pub fn pos_value(&self, mut square: usize, is_endgame: bool) -> i32 {
        if self.c == PieceColor::White {
            square = 63 - square;
        }

        match self.t {
            PieceType::Pawn => PAWN_VALUES[square],
            PieceType::Knight => KNIGHT_VALUES[square],
            PieceType::Bishop => BISHOP_VALUES[square],
            PieceType::Rook => ROOK_VALUES[square],
            PieceType::Queen => QUEEN_VALUES[square],
            PieceType::King => {
                if is_endgame {
                    KING_ENDGAME_VALUES[square]
                } else {
                    KING_VALUES[square]
                }
            }
        }
    }
}

impl PieceType {
    fn mat_value(&self) -> i32 {
        match self {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 0,
        }
    }
}

impl ChessState {
    pub fn static_eval(&self) -> i32 {
        let mut my_material_value = 0;
        let mut opp_material_value = 0;
        for i in 0..64 {
            if let Some(piece) = self.pieces[i] {
                if piece.c == self.turn {
                    my_material_value += piece.t.mat_value();
                } else {
                    opp_material_value += piece.t.mat_value();
                }
            }
        }
        let material_heu = my_material_value - opp_material_value;

        let is_endgame = (my_material_value + opp_material_value) <= 1600;

        let mut my_positional_value = 0;
        let mut opp_positional_value = 0;
        for i in 0..64 {
            if let Some(piece) = self.pieces[i] {
                if piece.c == self.turn {
                    my_positional_value += piece.pos_value(i, is_endgame);
                } else {
                    opp_positional_value += piece.pos_value(i, is_endgame);
                }
            }
        }
        let positional_heu = my_positional_value - opp_positional_value;

        let mut king_safety_heu = 0;
        if self.check {
            king_safety_heu -= 50;
        }

        if !is_endgame {
            king_safety_heu += (self.queen_castle[self.turn] as i32
                + self.king_castle[self.turn] as i32
                - self.queen_castle[self.turn.oppo()] as i32
                - self.king_castle[self.turn.oppo()] as i32)
                * 10;
        }

        material_heu + positional_heu + king_safety_heu
    }

    pub fn clock_factor(&self, score: i32) -> i32 {
        if self.halfmove_clock >= 50 {
            return 0;
        }

        let halfmove_factor = 1. / f32::exp(f32::powi(self.halfmove_clock as f32 / 40., 4));
        (score as f32 * halfmove_factor).round() as i32
    }
}

impl Move {
    pub fn static_eval(&self) -> i32 {
        let mut v = 0;

        if let Some(t) = self.capture {
            v += i32::max(100, t.mat_value() - self.pt.mat_value());
        }

        if self.check {
            v += 50;
        }

        if self.castle_king || self.castle_queen {
            v += 20;
        }

        if let Some(t) = self.promote_to {
            v += t.mat_value();
        }

        // TODO: Check if square is attacked by pawn

        v
    }
}
