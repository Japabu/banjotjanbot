use std::time::{Duration, Instant};

use super::{
    book::Book,
    gen_moves::Move,
    transposition_table::{TranspositionEntry, TranspositionEntryType, TranspositionTable},
    ChessState, PieceColorArray,
};

const CHECKMATE_EVAL: i32 = 1000000;
const MAX_SEARCH_DURATION: Duration = Duration::from_secs(31536000);
const MAX_DEPTH: u32 = 200;

const _TURN_MULT: PieceColorArray<i32> = PieceColorArray([1, -1]);

struct Search {
    search_end_time: Instant,
}

#[derive(PartialEq, Eq)]
enum NodeType {
    Root,
    PV,
    Cut,
    Quiesce,
}

impl Search {
    fn search<const NODE_TYPE: NodeType>(&self, state: &mut ChessState, mut alpha: i32, mut beta: i32, depth_left: i32, ply: u32) -> i32 {
        // Respect draws by the 50 move rule and by threefold repetition
        if state.halfmove_clock >= 50 || state.is_draw_by_repetition {
            return 0;
        }

        // If we have reached the maximum depth do a quiesce search
        // The quiesce search then continues infinitely until no more captures are possible with a depth_left < 0
        if NODE_TYPE != NodeType::Quiesce && depth_left == 0 {
            return self.search::<{ NodeType::Quiesce }>(state, alpha, beta, depth_left - 1, ply + 1);
        }

        let start_alpha = alpha;
        let mut best_move = None;

        // If we are doing a quiesce search perform optimisations such as
        // standing pat and delta pruning
        if NODE_TYPE == NodeType::Quiesce {
            let stand_pat = state.static_eval();

            if stand_pat >= beta {
                return beta;
            }

            // Delta pruning
            let delta = 900;
            if stand_pat + delta < alpha {
                return alpha;
            }

            if alpha < stand_pat {
                alpha = stand_pat;
            }
        }

        // Check if the current state is in the transposition table
        if let Some(transposition_entry) = TranspositionTable::get(state.hash) {
            if transposition_entry.depth >= depth_left {
                match transposition_entry.entry_type {
                    TranspositionEntryType::Exact => return transposition_entry.score,
                    TranspositionEntryType::LowerBound => alpha = i32::max(alpha, transposition_entry.score),
                    TranspositionEntryType::UpperBound => beta = i32::min(beta, transposition_entry.score),
                }

                if alpha >= beta {
                    return transposition_entry.score;
                }
            }

            // Get the best move from the latest transposition table entry
            best_move = transposition_entry.best_move;
        }

        // Generate pseudo legal moves because we can easily check if a move was legal in the search
        let mut moves = state.gen_pseudo_legal_moves();

        // If we are doing a quiesce search then only look at captures
        if NODE_TYPE == NodeType::Quiesce {
            moves.retain(|m| m.capture.is_some());
        }

        // Sort moves by score and put the best move first
        moves.sort_by_cached_key(|m| if Some(*m) == best_move { CHECKMATE_EVAL } else { m.static_eval() });

        let mut had_legal_move = false;
        let mut pv = NODE_TYPE == NodeType::PV || NODE_TYPE == NodeType::Root;
        for m in moves.iter().rev() {
            if Instant::now() >= self.search_end_time {
                return 0;
            }

            state.make_move(m);
            if state.check[state.turn.opposite()] {
                // Skip illegal moves
                state.unmake_last_move();
                continue;
            }

            had_legal_move = true;

            let score = if NODE_TYPE == NodeType::Quiesce {
                -self.search::<{ NodeType::Quiesce }>(state, -beta, -alpha, depth_left - 1, ply + 1)
            } else {
                if pv {
                    pv = false;
                    // The first child node of a pv node is also a pv node
                    -self.search::<{ NodeType::PV }>(state, -beta, -alpha, depth_left - 1, ply + 1)
                } else {
                    // All other nodes are cut nodes
                    let mut score = -self.search::<{ NodeType::Cut }>(state, -alpha - 1, -alpha, depth_left - 1, ply + 1);

                    // Do a re-search if the score looks promising
                    if score > alpha && score < beta {
                        score = -self.search::<{ NodeType::PV }>(state, -beta, -alpha, depth_left - 1, ply + 1);
                    }
                    score
                }
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

        // If we are not doing a quiesce search and we have not had any legal moves then
        // it is either a mate or a stalemate
        if NODE_TYPE != NodeType::Quiesce && !had_legal_move {
            return match state.check[state.turn] {
                true => -CHECKMATE_EVAL + ply as i32,
                false => 0,
            };
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

    /// Extracts the best line of moves from the transposition table
    fn best_line(&self, state: &ChessState, mut depth: u32) -> Vec<Move> {
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
    pub fn eval(&mut self, max_depth: Option<u32>, max_duration: Option<Duration>) -> (i32, Vec<Move>) {
        let max_depth = max_depth.unwrap_or(MAX_DEPTH);
        let max_duration = max_duration.unwrap_or(MAX_SEARCH_DURATION);

        let mut best_res = (self.static_eval(), Vec::new());

        if max_depth == 0 {
            return best_res;
        }

        let search = Search {
            search_end_time: Instant::now() + max_duration,
        };

        let mut depth = 0;
        let alpha = -CHECKMATE_EVAL;
        let beta = CHECKMATE_EVAL;
        while depth <= max_depth {
            let res = search.search::<{ NodeType::Root }>(self, alpha, beta, depth as i32, 0);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_by_repetition_test() {
        TranspositionTable::init();

        let mut state = ChessState::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(state.is_draw_by_repetition, false);

        // 1. Nh3 Nh6
        let mv = state.get_move("g1h3").unwrap();
        state.make_move(&mv);

        let mv = state.get_move("g8h6").unwrap();
        state.make_move(&mv);
        assert_eq!(state.is_draw_by_repetition, false);

        // 2. Ng1 Ng8
        let mv = state.get_move("h3g1").unwrap();
        state.make_move(&mv);

        let (score, mv) = state.eval(Some(1), None);
        assert_eq!(mv[0], state.get_move("h6g8").unwrap());
        assert_eq!(score, 0);
    }
}
