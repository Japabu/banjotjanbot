use super::{gen_moves::Move, ChessState, Piece, PieceColor, PieceType};

#[rustfmt::skip]
const PAWN_VALUES_MIDDLEGAME: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     40,  50,  50,  50,  50,  50,  50,  40,
     10,  20,  20,  30,  30,  20,  20,  10,
      5,  10,  10,  25,  25,  10,  10,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
      5,  -5, -10,   0,   0, -10,  -5,   5, 
      5,  10,  10, -20, -20,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const PAWN_VALUES_ENDGAME: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     40,  50,  50,  50,  50,  50,  50,  40,
     10,  20,  20,  30,  30,  20,  20,  10,
      5,  10,  10,  25,  25,  10,  10,   5,
      0,   0,   0,  20,  20,   0,   0,   0,
     -3,  -5,  -5,   0,   0,  -5,  -5,  -3, 
     -5, -10, -10, -20, -20, -10, -10,  -5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const KNIGHT_VALUES: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const BISHOP_VALUES: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const ROOK_VALUES: [i32; 64] = [
     0,  0,  0,  5,  5,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_VALUES: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10,   0,   0,  0,  0,   0,   0, -10,
    -10,   0,   5,  5,  5,   5,   0, -10,
     -5,   0,   5,  5,  5,   5,   0, -5,
      0,   0,   5,  5,  5,   5,   0, -5,
    -10,   5,   5,  5,  5,   5,   0, -10,
    -10,   0,   5,  0,  0,   0,   0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_VALUES_MIDDLEGAME: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20,
];

#[rustfmt::skip]
const KING_VALUES_ENDGAME: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -50, -30, -20, -10, -10, -20, -30, -50,
];

pub const LVA_MVV: [[i32; 6]; 6] = [
    [420, 220, 320, 520, 620, 120], // r captures r,n,b,q,k,p
    [440, 240, 340, 540, 640, 140], // n captures r,n,b,q,k,p
    [430, 230, 330, 530, 630, 130], // b captures r,n,b,q,k,p
    [410, 210, 310, 510, 610, 110], // q captures r,n,b,q,k,p
    [400, 200, 300, 500, 600, 100], // k captures r,n,b,q,k,p
    [450, 250, 350, 550, 650, 150], // p captures r,n,b,q,k,p
];

impl Piece {
    pub fn pos_value(&self, mut square: usize, is_endgame: bool) -> i32 {
        if self.c == PieceColor::White {
            square = 63 - square;
        }

        match self.t {
            PieceType::Rook => ROOK_VALUES[square],
            PieceType::Knight => KNIGHT_VALUES[square],
            PieceType::Bishop => BISHOP_VALUES[square],
            PieceType::Queen => QUEEN_VALUES[square],
            PieceType::King => {
                if is_endgame {
                    KING_VALUES_ENDGAME[square]
                } else {
                    KING_VALUES_MIDDLEGAME[square]
                }
            }
            PieceType::Pawn => {
                if is_endgame {
                    PAWN_VALUES_ENDGAME[square]
                } else {
                    PAWN_VALUES_MIDDLEGAME[square]
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
            PieceType::King => 20000,
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

        let total_material = my_material_value + opp_material_value;
        let material_heu = my_material_value - opp_material_value;

        let is_endgame = total_material <= 1600 + 2 * PieceType::King.mat_value();
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
                - self.queen_castle[self.turn.opposite()] as i32
                - self.king_castle[self.turn.opposite()] as i32)
                * 50;
        }

        material_heu + positional_heu + king_safety_heu
    }
}

impl Move {
    pub fn static_eval(&self) -> i32 {
        let mut v = 0;

        if let Some(t) = self.capture {
            v += LVA_MVV[self.pt as usize][t as usize];
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
    fn king_activity_midgame_test() {
        let state = ChessState::from_fen("7k/4p3/5q2/3Q4/4K3/4P3/8/8 w - - 0 1").unwrap();
        let eval_0 = state.static_eval();

        let state = ChessState::from_fen("7k/4p3/5q2/3Q4/8/4P3/8/K7 w - - 0 1").unwrap();
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
            PieceType::King,
            PieceType::Pawn,
        ] {
            for victim in [
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Knight,
                PieceType::Queen,
                PieceType::Pawn,
            ] {
                let (higher_value, lower_value) = if attacker.mat_value() > victim.mat_value() {
                    (attacker, victim)
                } else {
                    (victim, attacker)
                };

                assert!(
                    LVA_MVV[lower_value as usize][higher_value as usize]
                        >= LVA_MVV[higher_value as usize][lower_value as usize],
                    "{:?} captures {:?}",
                    attacker,
                    victim
                );
            }
        }
    }
}
