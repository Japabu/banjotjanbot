use std::time::{Duration, Instant};

use super::{
    book::Book,
    gen_moves::Move,
    transposition_table::{TranspositionEntry, TranspositionEntryType, TranspositionTable},
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

#[derive(PartialEq, Eq)]
enum NodeType {
    Root,
    PV,
    Cut,
}

impl Search {
    fn search<const NODE_TYPE: NodeType>(
        &self,
        state: &mut ChessState,
        mut alpha: i32,
        mut beta: i32,
        depth_left: u8,
    ) -> i32 {
        let mut moves = state.gen_moves();
        if moves.is_empty() {
            return match state.check {
                true => (self.start_depth - depth_left) as i32 - CHECKMATE_EVAL,
                false => 0,
            };
        }

        if state.halfmove_clock >= 50 {
            return 0;
        }

        if depth_left == 0 {
            return self.quiesce(state, alpha, beta);
        }

        let start_alpha = alpha;
        let mut best_move = None;

        if let Some(transposition_entry) = TranspositionTable::get(state.hash) {
            if transposition_entry.depth >= depth_left {
                match transposition_entry.entry_type {
                    TranspositionEntryType::Exact => return transposition_entry.score,
                    TranspositionEntryType::LowerBound => {
                        alpha = i32::max(alpha, transposition_entry.score)
                    }
                    TranspositionEntryType::UpperBound => {
                        beta = i32::min(beta, transposition_entry.score)
                    }
                }

                if alpha >= beta {
                    return transposition_entry.score;
                }
            }

            best_move = transposition_entry.best_move;
        }

        moves.sort_by_cached_key(|m| {
            if Some(*m) == best_move {
                CHECKMATE_EVAL
            } else {
                m.static_eval()
            }
        });

        let mut pv = NODE_TYPE != NodeType::Cut;
        for m in moves.iter().rev() {
            if Instant::now() >= self.search_end_time {
                return 0;
            }

            state.make_move(m);

            let score = if pv {
                pv = false;
                -self.search::<{ NodeType::PV }>(state, -beta, -alpha, depth_left - 1)
            } else {
                let mut score =
                    -self.search::<{ NodeType::Cut }>(state, -alpha - 1, -alpha, depth_left - 1);
                if score > alpha && score < beta {
                    score = -self.search::<{ NodeType::PV }>(state, -beta, -alpha, depth_left - 1);
                }
                score
            };

            state.unmake_last_move();

            if score >= beta {
                alpha = score;
                break;
            }
            if score > alpha {
                alpha = score;
                best_move = Some(*m);
            }
        }

        TranspositionTable::set(
            state.hash,
            TranspositionEntry {
                key: state.hash,
                entry_type: if alpha <= start_alpha {
                    TranspositionEntryType::UpperBound
                } else if alpha >= beta {
                    TranspositionEntryType::LowerBound
                } else {
                    TranspositionEntryType::Exact
                },
                depth: depth_left,
                score: alpha,
                best_move,
            },
        );

        alpha
    }

    fn quiesce(&self, state: &mut ChessState, mut alpha: i32, beta: i32) -> i32 {
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
            if Instant::now() >= self.search_end_time {
                return 0;
            }

            state.make_move(m);
            let score = -self.quiesce(state, -beta, -alpha);
            state.unmake_last_move();

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

        let mut state = state.clone();

        while depth > 0 && let Some(t) = TranspositionTable::get(state.hash) && let Some(m) = t.best_move {
            moves.push(m);
            state.make_move(&m);

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
    pub fn eval(
        &mut self,
        max_depth: Option<u8>,
        max_duration: Option<Duration>,
    ) -> (i32, Vec<Move>) {
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
        let mut alpha = -CHECKMATE_EVAL;
        let mut beta = CHECKMATE_EVAL;
        while depth <= max_depth {
            search.start_depth = depth;
            let res = search.search::<{ NodeType::Root }>(self, alpha, beta, depth);

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

    pub fn find_book_move(&mut self) -> Option<Move> {
        if let Some(book_move) = Book::get(self.hash) {
            Some(
                *self
                    .gen_moves()
                    .iter()
                    .find(|m| m.to_string() == book_move)
                    .expect("Book move is not in moves!"),
            )
        } else {
            None
        }
    }
}
