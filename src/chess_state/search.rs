use std::time::{Duration, Instant};

use super::{
    gen_moves::Move,
    transposition_table::{TranspositionEntry, TranspositionTable},
    ChessState, PieceColorArray,
};

const CHECKMATE_EVAL: i32 = -1000000;
const MAX_SEARCH_DURATION: Duration = Duration::from_secs(31536000);
const MAX_DEPTH: u32 = 200;

const _TURN_MULT: PieceColorArray<i32> = PieceColorArray([1, -1]);

struct Search {
    search_end_time: Instant,
    start_depth: u32,
}

impl Search {
    

    fn par_pvs(&mut self, state: &ChessState, mut alpha: i32, beta: i32, depth_left: u32) -> i32 {
        if state.halfmove_clock >= 50 {
            return 0;
        }

        let transposition_entry = TranspositionTable::get(state);
        // let transposition_entry: Option<TranspositionEntry> = None;
        if let Some(t) = transposition_entry
        && t.depth >= depth_left {
                return state.clock_factor(t.score);
        }

        if depth_left <= 0 {
            // return self.quiesce(state, alpha, beta);
            let static_score = state.static_eval();

            TranspositionTable::set(
                &state,
                TranspositionEntry {
                    key: state.hash,
                    depth: depth_left,
                    score: static_score,
                    best_move: None,
                },
            );

            return state.clock_factor(static_score);
        }

        let mut moves = state.gen_moves();
        if moves.len() == 0 {
            return match state.check {
                true => CHECKMATE_EVAL + (self.start_depth - depth_left) as i32,
                false => 0,
            };
        }
        moves.sort_by_cached_key(|m| m.static_eval());

        let mut best_move = if let Some(t) = transposition_entry
                            && let Some(m) = t.best_move
                            && let Some(i) = moves.iter().rposition(|x| *x == m) {
            moves.remove(i)
        } else {
            moves.pop().unwrap()
        };

        let mut s = *state;
        s.make_move(&best_move);
        let mut best_score = -self.par_pvs(&s, -beta, -alpha, depth_left - 1);

        if best_score > alpha {
            if best_score >= beta {
                return best_score;
            }
            alpha = best_score;
        }

        for m in moves.iter().rev() {
            if Instant::now() >= self.search_end_time {
                return 0;
            }

            let mut s = *state;
            s.make_move(m);
            let mut score = -self.par_pvs(&s, -alpha - 1, -alpha, depth_left - 1);
            if score > alpha && score < beta {
                score = -self.par_pvs(&s, -beta, -alpha, depth_left - 1);
                if score > alpha {
                    alpha = score;
                }
            }
            if score > best_score {
                if score >= beta {
                    return score;
                }

                best_move = *m;
                best_score = score;
            }
        }

        TranspositionTable::set(
            state,
            TranspositionEntry {
                key: state.hash,
                depth: depth_left,
                score: best_score,
                best_move: Some(best_move),
            },
        );

        best_score
    }

    fn best_line(&self, state: &ChessState, mut depth: u32) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut s = *state;
        while depth > 0 && let Some(t) = TranspositionTable::get(&s) && let Some(m) = t.best_move {
            moves.push(m);
            s.make_move(&m);

            depth-=1;
        }

        moves
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
        };

        let mut depth = 0;
        while depth <= max_depth {
            search.start_depth = depth;
            let res = search.par_pvs(self, CHECKMATE_EVAL, -CHECKMATE_EVAL, depth);

            if Instant::now() >= search.search_end_time {
                break;
            }

            let line = search.best_line(self, depth);
            println!("{} {} {}", depth, res, fmt_moves(&line));

            best_res = (res, line);
            depth += 1;
        }

        best_res
    }
}
