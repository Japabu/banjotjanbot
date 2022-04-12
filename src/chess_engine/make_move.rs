use super::{gen_moves::Move, with_offset, zobrist::Zobrist, ChessState, Piece, PieceColor, PieceColorArray, PieceType};

const CASTLE_OFFSET: PieceColorArray<u8> = PieceColorArray([0, 7 * 8]);
const QUEEN_CASTLE_SQUARES: PieceColorArray<[u8; 2]> = PieceColorArray([[0, 4], [0 + 7 * 8, 4 + 7 * 8]]);
const KING_CASTLE_SQUARES: PieceColorArray<[u8; 2]> = PieceColorArray([[4, 7], [4 + 7 * 8, 7 + 7 * 8]]);

#[derive(Clone, Copy)]
pub struct Unmove {
    old_en_passant_target: Option<u8>,
    old_halfmove_clock: u8,
    castle_queen: bool,
    castle_king: bool,
    old_check: PieceColorArray<bool>,
    old_queen_castle: PieceColorArray<bool>,
    old_king_castle: PieceColorArray<bool>,
    old_king_pos: u8,
    piece_type: PieceType,
    from: u8,
    to: u8,
    captured: Option<PieceType>,
    old_hash: u64,
    en_passant: bool,
}

impl ChessState {
    pub fn make_move(&mut self, m: &Move) {
        self.unmove_stack.push(Unmove {
            old_en_passant_target: self.en_passant_target,
            old_halfmove_clock: self.halfmove_clock,
            castle_queen: m.castle_queen,
            castle_king: m.castle_king,
            old_check: self.check,
            old_queen_castle: self.queen_castle,
            old_king_castle: self.king_castle,
            old_king_pos: self.king_pos[self.turn],
            piece_type: m.piece_type,
            from: m.from,
            to: m.to,
            captured: m.capture,
            old_hash: self.hash,
            en_passant: m.en_passant,
        });

        self.hash = Zobrist::inc_update(self.hash, self, m);

        // Clear en passant square
        self.en_passant_target = None;

        // Only set en passant target if there is an enemy pawn ready to perform it
        if m.new_en_passant_target.is_some() {
            for i in [-1, 1].iter().filter_map(|o| with_offset(m.to, *o)) {
                match self.pieces[i as usize] {
                    Some(Piece { c, t: PieceType::Pawn }) if c == self.turn.opposite() => {
                        self.en_passant_target = m.new_en_passant_target;
                        break;
                    }
                    _ => (),
                }
            }
        }

        if m.piece_type == PieceType::Pawn || m.capture.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.turn == PieceColor::Black {
            self.move_clock += 1;
        }

        if m.castle_queen {
            let offset = CASTLE_OFFSET[self.turn];
            self.pieces[0 + offset as usize] = None;
            self.pieces[2 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::King,
            });
            self.pieces[3 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::Rook,
            });
            self.pieces[4 + offset as usize] = None;
            self.queen_castle[self.turn] = false;
            self.king_castle[self.turn] = false;
            self.king_pos[self.turn] = 2 + offset;
            self.turn = self.turn.opposite();
            self.update_check();
            self.increment_current_position_counter_and_update_draw_by_repetition();
            return;
        } else if m.castle_king {
            let offset = CASTLE_OFFSET[self.turn];
            self.pieces[4 + offset as usize] = None;
            self.pieces[5 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::Rook,
            });
            self.pieces[6 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::King,
            });
            self.pieces[7 + offset as usize] = None;
            self.queen_castle[self.turn] = false;
            self.king_castle[self.turn] = false;
            self.king_pos[self.turn] = 6 + offset;
            self.turn = self.turn.opposite();
            self.update_check();
            self.increment_current_position_counter_and_update_draw_by_repetition();
            return;
        }

        for color in [PieceColor::White, PieceColor::Black] {
            self.queen_castle[color] &= !QUEEN_CASTLE_SQUARES[color].contains(&m.from) && !QUEEN_CASTLE_SQUARES[color].contains(&m.to);

            self.king_castle[color] &= !KING_CASTLE_SQUARES[color].contains(&m.from) && !KING_CASTLE_SQUARES[color].contains(&m.to);
        }

        if m.piece_type == PieceType::King {
            self.king_pos[self.turn] = m.to;
        }

        self.pieces[m.to as usize] = self.pieces[m.from as usize];
        self.pieces[m.from as usize] = None;

        if m.en_passant {
            self.pieces[(m.to as i8
                + match self.turn {
                    PieceColor::White => -8,
                    PieceColor::Black => 8,
                }) as usize] = None;
        }

        if let Some(t) = m.promote_to {
            self.pieces[m.to as usize] = Some(Piece { c: self.turn, t });
        }

        self.turn = self.turn.opposite();
        self.update_check();
        self.increment_current_position_counter_and_update_draw_by_repetition();
    }

    pub fn unmake_last_move(&mut self) {
        self.decrement_current_position_counter();

        self.turn = self.turn.opposite();

        let unmove = self.unmove_stack.pop().unwrap();
        self.en_passant_target = unmove.old_en_passant_target;
        self.halfmove_clock = unmove.old_halfmove_clock;
        self.check = unmove.old_check;
        self.queen_castle = unmove.old_queen_castle;
        self.king_castle = unmove.old_king_castle;
        self.king_pos[self.turn] = unmove.old_king_pos;
        self.hash = unmove.old_hash;

        if self.turn == PieceColor::Black {
            self.move_clock -= 1;
        }

        if unmove.castle_queen {
            let offset = CASTLE_OFFSET[self.turn];
            self.pieces[0 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::Rook,
            });
            self.pieces[2 + offset as usize] = None;
            self.pieces[3 + offset as usize] = None;
            self.pieces[4 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::King,
            });
        } else if unmove.castle_king {
            let offset = CASTLE_OFFSET[self.turn];
            self.pieces[4 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::King,
            });
            self.pieces[5 + offset as usize] = None;
            self.pieces[6 + offset as usize] = None;
            self.pieces[7 + offset as usize] = Some(Piece {
                c: self.turn,
                t: PieceType::Rook,
            });
        }

        if unmove.en_passant {
            self.pieces[(unmove.to as i8
                + match self.turn {
                    PieceColor::White => -8,
                    PieceColor::Black => 8,
                }) as usize] = Some(Piece {
                c: self.turn.opposite(),
                t: PieceType::Pawn,
            });
            self.pieces[unmove.to as usize] = None;
        } else {
            self.pieces[unmove.to as usize] = match unmove.captured {
                Some(t) => Some(Piece { c: self.turn.opposite(), t }),
                None => None,
            };
        }

        self.pieces[unmove.from as usize] = Some(Piece {
            c: self.turn,
            t: unmove.piece_type,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{gen_moves::Move, ChessState};

    fn find_move(moves: &[Move], m: &str) -> Option<Move> {
        moves.iter().find(|mv| mv.to_string() == m).cloned()
    }

    #[test]
    fn unmake_move_e2e_test4() {
        let mut state = ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e2e4").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap().hash
        );
    }

    #[test]
    fn unmake_move_castle_test() {
        let mut state = ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e1c1").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/2KR3R b kq - 1 1").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap().hash
        );

        let mut state = ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e1g1").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R4RK1 b kq - 1 1").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap().hash
        );

        let mut state = ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e8c8").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("2kr3r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQ - 1 2").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap().hash
        );

        let mut state = ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e8g8").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("r4rk1/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQ - 1 2").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap().hash
        );
    }

    #[test]
    fn unmake_move_enpassant_test() {
        let mut state = ChessState::from_fen("r3k2r/pppp1ppp/8/8/4p3/8/PPPPPPPP/R3K2R w KQkq - 0 3").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "d2d4").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/R3K2R b KQkq d3 0 3").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppp1ppp/8/8/4p3/8/PPPPPPPP/R3K2R w KQkq - 0 3").unwrap().hash
        );

        let mut state = ChessState::from_fen("r3k2r/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/R3K2R b KQkq d3 0 3").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e4d3").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppp1ppp/8/8/8/3p4/PPP1PPPP/R3K2R w KQkq - 0 4").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/R3K2R b KQkq d3 0 3").unwrap().hash
        );

        let mut state = ChessState::from_fen("r3k2r/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/R3K2R b KQkq d3 0 3").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "d7d6").unwrap());

        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/ppp2ppp/3p4/8/3Pp3/8/PPP1PPPP/R3K2R w KQkq - 0 4").unwrap().hash
        );

        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/R3K2R b KQkq d3 0 3").unwrap().hash
        );
    }

    #[test]
    fn unmake_move_test() {
        let mut state = ChessState::from_fen("r3k2r/pppp1ppp/8/4p3/8/8/PPPPPPPP/R3K2R b KQkq - 0 2").unwrap();
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e5e4").unwrap());
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "d2d4").unwrap());
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e4d3").unwrap());
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "c2d3").unwrap());
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppp1ppp/8/8/8/3P4/PP2PPPP/R3K2R b KQkq - 0 4").unwrap().hash
        );

        state.unmake_last_move();
        state.unmake_last_move();
        state.unmake_last_move();
        state.unmake_last_move();
        assert_eq!(
            state.hash,
            ChessState::from_fen("r3k2r/pppp1ppp/8/4p3/8/8/PPPPPPPP/R3K2R b KQkq - 0 2").unwrap().hash
        );

        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e8e7").unwrap());
        assert_eq!(
            state.hash,
            ChessState::from_fen("r6r/ppppkppp/8/4p3/8/8/PPPPPPPP/R3K2R w KQ - 1 3").unwrap().hash
        );
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e2e4").unwrap());
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e7e6").unwrap());
        let moves = state.gen_moves();
        state.make_move(&find_move(&moves, "e1e2").unwrap());
        assert_eq!(
            state.hash,
            ChessState::from_fen("r6r/pppp1ppp/4k3/4p3/4P3/8/PPPPKPPP/R6R b - - 2 4").unwrap().hash
        );
    }

    #[test]
    fn draw_by_repetition_test() {
        let mut state = ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let mv = state.get_move("b1c3").unwrap();
        state.make_move(&mv);
        let mv = state.get_move("b8c6").unwrap();
        state.make_move(&mv);
        assert!(!state.is_draw_by_repetition);
        let mv = state.get_move("c3b1").unwrap();
        state.make_move(&mv);
        let mv = state.get_move("c6b8").unwrap();
        state.make_move(&mv);
        assert!(state.is_draw_by_repetition);
    }
}
