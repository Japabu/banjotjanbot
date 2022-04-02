use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, sync::RwLock};

use crate::chess_state::{PieceColorArray, gen_moves::with_offset};

use super::{gen_moves::Move, ChessState, Piece, PieceColor, PieceType};

const CHECKMATE_EVAL: i32 = -1000000;

const PAWN_VALUES: PieceColorArray<[i32; 8]> = PieceColorArray([
    [0, 100, 105, 106, 107, 150, 250, 0],
    [0, 250, 150, 107, 106, 105, 100, 0],
]);

#[derive(Hash, PartialEq, Eq)]
struct TranspositionKey {
    pieces: [Option<Piece>; 64],
    turn: PieceColor,
    king_castle: PieceColorArray<bool>,
    queen_castle: PieceColorArray<bool>,
    en_passant_target: Option<usize>,
}

impl TranspositionKey {
    fn new(state: &ChessState) -> Self {
        TranspositionKey {
            pieces: state.pieces,
            turn: state.turn,
            king_castle: state.king_castle,
            queen_castle: state.queen_castle,
            en_passant_target: state.en_passant_target,
        }
    }
}

lazy_static! {
    static ref TRANSPOSITION_TABLE: RwLock<HashMap<TranspositionKey, i32>> =
        RwLock::new(HashMap::new());
}

fn set_transposition(state: &ChessState, value: i32) -> i32 {
    return value;
    TRANSPOSITION_TABLE
        .write()
        .unwrap()
        .insert(TranspositionKey::new(state), value);
    value
}

fn get_transposition(state: &ChessState) -> Option<i32> {
    return None;
    TRANSPOSITION_TABLE
        .read()
        .unwrap()
        .get(&TranspositionKey::new(state))
        .cloned()
}

impl Piece {
    fn heu_value(&self, pos: usize) -> i32 {
        match self.t {
            PieceType::Queen => 900,
            PieceType::Rook => 400,
            PieceType::Knight | PieceType::Bishop => 300,
            PieceType::Pawn => PAWN_VALUES[self.c][pos / 8],
            PieceType::King => 0,
        }
    }
}

impl Move {
    fn heu_value(&self) -> i32 {
        let mut v = 0;

        if self.capture {
            v += 10;
        }

        if self.check {
            v += 5;
        }

        if self.castle_king || self.castle_queen {
            v += 2;
        }

        v += match self.promote_to {
            Some(PieceType::Queen) => 15,
            Some(PieceType::Rook) => 10,
            Some(PieceType::Bishop) => 9,
            Some(PieceType::Knight) => 9,
            _ => 0,
        };

        -v
    }
}

impl ChessState {
    const fn turn_mult(&self) -> i32 {
        return match self.turn {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };
    }

    fn quiesce(&self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = self.heu_eval();

        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        for m in self.gen_moves().iter().filter(|m| m.capture) {
            let mut s = *self;
            s.make_move(&m);
            let score = -s.quiesce(-beta, -alpha);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn negamax(&self, start_depth: i32, depth: i32, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return self.heu_eval();
            return self.quiesce(alpha, beta);
        }

        let mut moves = self.gen_moves();

        if moves.len() == 0 {
            return match self.check {
                true => CHECKMATE_EVAL + (start_depth - depth) as i32,
                false => 0,
            };
        }

        moves.sort_by_cached_key(|m| m.heu_value());

        let mut value = CHECKMATE_EVAL;
        for m in moves {
            let mut s = *self;
            s.make_move(&m);
            value = i32::max(value, -s.negamax(start_depth, depth - 1, -beta, -alpha));
            alpha = i32::max(alpha, value);
            if alpha >= beta {
                break;
            }
        }

        value
    }

    fn heu_eval(&self) -> i32 {
        if let Some(value) = get_transposition(self) {
            return value;
        }

        const A: PieceColorArray<i32> = PieceColorArray([1, -1]);
        let material_heu = self
            .pieces
            .iter()
            .enumerate()
            .map(|x| x.1.map_or(0, |p| p.heu_value(x.0) * A[p.c]))
            .sum::<i32>()
            * A[self.turn];

        let mut s = *self;
        s.turn = s.turn.oppo();
        let move_heu = (self.gen_moves().len() - s.gen_moves().len()) as i32 * 10;

        let castle_heu = (self.queen_castle[self.turn] as i32 + self.king_castle[self.turn] as i32
            - self.queen_castle[self.turn.oppo()] as i32
            - self.king_castle[self.turn.oppo()] as i32)
            * 10;

        let material_count = self
            .pieces
            .iter()
            .enumerate()
            .map(|x| x.1.map_or(0, |p| p.heu_value(x.0)))
            .sum::<i32>();

        let endgame = material_count <= 16;
        
        let mut king_safety_heu = 0;
        if endgame {

        } else {
            for offset in PieceType::King.offsets() {
                if let Some(n) = with_offset(self.king_pos[self.turn], *offset) {
                    if let Some(p) = self.pieces[n] {
                        if p.c == self.turn {
                            king_safety_heu += 5;
                        }
                    }
                } else {
                    king_safety_heu += 5;
                }

                if let Some(n) = with_offset(self.king_pos[self.turn.oppo()], *offset) {
                    if let Some(p) = self.pieces[n] {
                        if p.c == self.turn.oppo() {
                            king_safety_heu -= 5;
                        }
                    }
                } else {
                    king_safety_heu -= 5;
                }
            }
        }

        if self.check {
            king_safety_heu -= 50;
        }

        set_transposition(self, material_heu + castle_heu + move_heu + king_safety_heu)
    }

    /// Evaluates the position relative to the current player
    /// This means, the better the position for self.turn the bigger this value
    fn eval(&self, depth: u32) -> i32 {
        self.negamax(depth as i32, depth as i32, CHECKMATE_EVAL, -CHECKMATE_EVAL)
    }

    /// Evaluates the position absolutely (eval>0 favors white, eval<0 favors black)
    pub fn absolute_eval(&self, depth: u32) -> i32 {
        match self.turn {
            PieceColor::White => self.eval(depth),
            PieceColor::Black => -self.eval(depth),
        }
    }

    /// Returns the best move for self.turn and the absolute eval after the move is done
    pub fn find_best_move(&self, depth: u32) -> (Option<Move>, i32) {
        let moves = self.gen_moves();
        if moves.len() == 0 {
            return match self.check {
                true => (None, CHECKMATE_EVAL * self.turn_mult()),
                false => (None, 0),
            };
        }

        let (best_move, best_value) = moves
            .par_iter()
            .map(|m| {
                let mut s = *self;
                s.make_move(m);
                (m, -s.eval(depth - 1))
            })
            .max_by_key(|x| x.1)
            .unwrap();

        (Some(*best_move), best_value * self.turn_mult())
    }
}
