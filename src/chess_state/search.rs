use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::time::{Duration, Instant};

use super::{gen_moves::Move, ChessState, PieceColor, PieceColorArray};

const CHECKMATE_EVAL: i32 = -1000000;
const MAX_SEARCH_DURATION: Duration = Duration::from_secs(31536000);
const MAX_DEPTH: u32 = 200;

const TURN_MULT: PieceColorArray<i32> = PieceColorArray([1, -1]);

struct Search {
    search_end_time: Instant,
    start_depth: u32,
    principal_moves: Vec<Move>,
}

impl Search {
    fn par_pvs(&mut self, state: &ChessState, mut alpha: i32, beta: i32, depth_left: u32) -> i32 {
        if depth_left <= 0 {
            return state.static_eval();
            // return self.quiesce(state, alpha, beta);
        }

        let mut moves = state.gen_moves();
        if moves.len() == 0 {
            return match state.check {
                true => CHECKMATE_EVAL + (self.start_depth - depth_left) as i32,
                false => 0,
            };
        }

        let mut best_score = CHECKMATE_EVAL;

        let depth = self.start_depth - depth_left;
        if self.principal_moves.len() as u32 > depth {
            let principal_move = self.principal_moves[depth as usize];

            if let Some(principal_move_idex) = moves.iter().position(|m| *m == principal_move) {
                moves.swap_remove(principal_move_idex);

                let mut s = *state;
                s.make_move(&principal_move);
                best_score = -self.par_pvs(&s, -beta, -alpha, depth_left - 1);

                if best_score > alpha {
                    if best_score >= beta {
                        return best_score;
                    }
                    alpha = best_score;
                }
            }
        }

        moves.sort_by_cached_key(|m| m.static_eval());
        for m in moves {
            if Instant::now() >= self.search_end_time {
                return 0;
            }

            let mut s = *state;
            s.make_move(&m);
            let mut score = -self.par_pvs(&s, -alpha - 1, -alpha, depth_left - 1);
            if score > alpha && score < beta {
                score = -self.par_pvs(&s, -beta, -alpha, depth_left - 1);
                if score > alpha {
                    alpha = score;
                    if self.principal_moves.len() <= depth as usize {
                        self.principal_moves.push(m);
                    } else {
                        self.principal_moves[depth as usize] = m;
                    }
                }
            }
            if score > best_score {
                if score >= beta {
                    return score;
                }
                best_score = score;
            }
        }

        best_score
    }
}

fn fmt_moves(moves: &[Move]) -> String {
    let mut s = String::new();
    for m in moves {
        s.push_str(&format!("{} ", m));
    }
    s
}

impl ChessState {
    pub fn eval(&self, max_depth: Option<u32>, max_duration: Option<Duration>) -> (i32, Vec<Move>) {
        let max_depth = max_depth.unwrap_or(MAX_DEPTH);
        let max_duration = max_duration.unwrap_or(MAX_SEARCH_DURATION);

        let mut best_res = (self.static_eval(), Vec::new());

        if max_depth == 0 {
            return best_res;
        }

        let mut search = Search {
            search_end_time: Instant::now() + max_duration,
            start_depth: 0,
            principal_moves: Vec::new(),
        };

        let mut depth = 1;
        while depth <= max_depth {
            search.start_depth = depth;
            let res = search.par_pvs(self, CHECKMATE_EVAL, -CHECKMATE_EVAL, depth);

            if Instant::now() >= search.search_end_time {
                break;
            }

            best_res = (res, search.principal_moves.clone());
            depth += 1;

            println!("{} {}", depth, fmt_moves(&search.principal_moves));
        }

        best_res
    }
}
