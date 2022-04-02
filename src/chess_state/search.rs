use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use super::{gen_moves::Move, ChessState, PieceColor, PieceType};

const CHECKMATE_EVAL: i32 = -1000000;

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

    // fn quiesce(&self, mut alpha: i32, beta: i32, shutdown: &AtomicBool) -> i32 {
    //     let stand_pat = self.static_eval();

    //     if stand_pat >= beta {
    //         return beta;
    //     }
    //     if alpha < stand_pat {
    //         alpha = stand_pat;
    //     }

    //     for m in self.gen_moves().iter().filter(|m| m.capture) {
    //         let mut s = *self;
    //         s.make_move(&m);
    //         let score = -s.quiesce(-beta, -alpha, shutdown);

    //         if score >= beta {
    //             return beta;
    //         }
    //         if score > alpha {
    //             alpha = score;
    //         }
    //     }

    //     alpha
    // }

    fn negamax(
        &self,
        start_depth: i32,
        depth: i32,
        mut alpha: i32,
        beta: i32,
        shutdown: &AtomicBool,
    ) -> i32 {
        if depth == 0 || shutdown.load(Ordering::Relaxed) {
            return self.static_eval();
            // return self.quiesce(alpha, beta, shutdown);
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
            value = i32::max(
                value,
                -s.negamax(start_depth, depth - 1, -beta, -alpha, shutdown),
            );
            alpha = i32::max(alpha, value);
            if alpha >= beta {
                break;
            }
        }

        value
    }

    /// Evaluates the position relative to the current player
    /// This means, the better the position for self.turn the bigger this value
    fn eval(&self, depth: u32, shutdown: &AtomicBool) -> i32 {
        self.negamax(
            depth as i32,
            depth as i32,
            CHECKMATE_EVAL,
            -CHECKMATE_EVAL,
            shutdown,
        )
    }

    /// Evaluates the position absolutely (eval>0 favors white, eval<0 favors black)
    pub fn absolute_eval(&self, depth: u32) -> i32 {
        match self.turn {
            PieceColor::White => self.eval(depth, &AtomicBool::new(false)),
            PieceColor::Black => -self.eval(depth, &AtomicBool::new(false)),
        }
    }

    fn find_best_move_internal(&self, depth: u32, shutdown: &AtomicBool) -> (Option<Move>, i32) {
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
                (m, -s.eval(depth - 1, shutdown))
            })
            .max_by_key(|x| x.1)
            .unwrap();

        (Some(*best_move), best_value * self.turn_mult())
    }

    /// Returns the best move for self.turn and the absolute eval after the move is done
    pub fn find_best_move_with_depth(&self, depth: u32) -> (Option<Move>, i32) {
        self.find_best_move_internal(depth, &AtomicBool::new(false))
    }

    pub fn find_best_move_with_time(&self, seconds: u64) -> (Option<Move>, i32, u32) {
        let shutdown = Arc::new(AtomicBool::new(false));

        thread::scope(|s| {
            let worker = s.spawn(|| {
                let mut last_result = (None, 0, 0);
                let mut depth = 1;
                loop {
                    let result = self.find_best_move_internal(depth, &shutdown);
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }

                    println!(
                        "{} {} {}",
                        depth,
                        result
                            .0
                            .map_or_else(|| String::from("None"), |x| x.to_string()),
                        result.1
                    );

                    last_result = (result.0, result.1, depth);
                    depth += 1;
                }

                last_result
            });

            thread::sleep(Duration::from_secs(seconds));
            shutdown.store(true, Ordering::Relaxed);

            worker.join()
        })
        .unwrap()
    }
}
