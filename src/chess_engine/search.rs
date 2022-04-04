use std::time::{Duration, Instant};

use super::{
    gen_moves::Move,
    transposition_table::{TranspositionEntry, TranspositionTable},
    ChessState, PieceColorArray,
};

const CHECKMATE_EVAL: i32 = 1000000;
const MAX_SEARCH_DURATION: Duration = Duration::from_secs(31536000);
const MAX_DEPTH: u8 = 200;

const _TURN_MULT: PieceColorArray<i32> = PieceColorArray([1, -1]);

struct Search {
    search_end_time: Instant,
    start_depth: u8,
}

impl Search {
    fn pvs(&self, state: &ChessState, mut alpha: i32, beta: i32, depth_left: u8) -> i32 {
        if state.halfmove_clock >= 50 {
            return 0;
        }

        if depth_left == 0 {
            return self.quiesce(state, alpha, beta);
        }

        let mut best_move = None;

        let transposition_entry = TranspositionTable::get(state.hash);
        if let Some(t) = transposition_entry {
            if t.depth >= depth_left {
                return t.score;
            }

            if let Some(m) = t.best_move {
                //Try best move from transposition table
                best_move = t.best_move;

                let mut s = *state;
                s.make_move(&m);
                let score = -self.pvs(&s, -beta, -alpha, depth_left - 1);
                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                    TranspositionTable::set(
                        state.hash,
                        TranspositionEntry {
                            key: state.hash,
                            depth: depth_left,
                            score,
                            best_move,
                        },
                    )
                }
            }
        }

        let mut moves = state.gen_moves();
        if let Some(m) = best_move {
            moves.iter().position(|x| *x == m).map_or_else(
                || panic!("best move not in moves"),
                |i| moves.swap_remove(i),
            );
        }

        if moves.is_empty() {
            return match state.check {
                true => (self.start_depth - depth_left) as i32 - CHECKMATE_EVAL,
                false => 0,
            };
        }
        moves.sort_by_cached_key(|m| m.static_eval());

        for m in moves.iter().rev() {
            let mut s = *state;
            s.make_move(m);

            let mut score = -self.zws(&s, -alpha, depth_left - 1);
            if score > alpha {
                // in fail-soft ... && score < beta ) is common
                score = -self.pvs(&s, -beta, -alpha, depth_left - 1); // re-search
            }

            if score >= beta {
                return beta; // fail-hard beta-cutoff
            }
            if score > alpha {
                alpha = score; // alpha acts like max in MiniMax
                best_move = Some(*m);
            }
        }

        TranspositionTable::set(
            state.hash,
            TranspositionEntry {
                key: state.hash,
                depth: depth_left,
                score: alpha,
                best_move: best_move,
            },
        );

        alpha
    }

    // fail-hard zero window search, returns either beta-1 or beta
    fn zws(&self, state: &ChessState, beta: i32, depth_left: u8) -> i32 {
        // alpha == beta - 1
        // this is either a cut- or all-node
        if depth_left == 0 {
            return self.quiesce(state, beta - 1, beta);
        }

        let mut moves = state.gen_moves();
        if moves.is_empty() {
            return match state.check {
                true => (self.start_depth - depth_left) as i32 - CHECKMATE_EVAL,
                false => 0,
            };
        }
        moves.sort_by_cached_key(|m| m.static_eval());

        for m in moves {
            let mut s = *state;
            s.make_move(&m);

            let score = -self.zws(&s, 1 - beta, depth_left - 1);
            if score >= beta {
                return beta; // fail-hard beta-cutoff
            }
        }
        beta - 1 // fail-hard, return alpha
    }

    fn quiesce(&self, state: &ChessState, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = state.static_eval();
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let mut moves = state.gen_moves();
        moves.retain(|m| m.capture.is_some());
        moves.sort_by_cached_key(|m| m.static_eval());

        for m in moves.iter().rev() {
            let mut s = *state;
            s.make_move(m);
            let score = -self.quiesce(&s, -beta, -alpha);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn best_line(&self, state: &ChessState, mut depth: u8) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut s = *state;
        while depth > 0 && let Some(t) = TranspositionTable::get(s.hash) && let Some(m) = t.best_move {
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
    pub fn eval(&self, max_depth: Option<u8>, max_duration: Option<Duration>) -> (i32, Vec<Move>) {
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
            let res = search.pvs(self, -CHECKMATE_EVAL, CHECKMATE_EVAL, depth);

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
