use crate::chess_engine::PieceColorArray;

use super::{with_offset, ChessState, Piece, PieceColor, PieceType};

impl PieceType {
    const fn is_sliding(&self) -> bool {
        match self {
            PieceType::Rook => true,
            PieceType::Knight => false,
            PieceType::Bishop => true,
            PieceType::Queen => true,
            PieceType::King => false,
            _ => false,
        }
    }

    pub const fn offsets(&self) -> &[i8] {
        match self {
            PieceType::Rook => &[-10, -1, 1, 10],
            PieceType::Knight => &[-21, -19, -12, -8, 8, 12, 19, 21],
            PieceType::Bishop => &[-11, -9, 9, 11],
            PieceType::Queen | PieceType::King => &[-11, -10, -9, -1, 1, 9, 10, 11],
            _ => &[],
        }
    }
}

#[derive(Clone, Copy, Eq, Debug)]
pub struct Move {
    pub piece_type: PieceType,
    pub from: u8,
    pub to: u8,
    pub promote_to: Option<PieceType>,
    pub new_en_passant_target: Option<u8>,
    pub castle_king: bool,
    pub castle_queen: bool,
    pub en_passant: bool,
    pub capture: Option<PieceType>,
}

impl Default for Move {
    fn default() -> Self {
        Move {
            from: 0,
            to: 0,
            promote_to: None,
            new_en_passant_target: None,
            castle_king: false,
            castle_queen: false,
            en_passant: false,
            piece_type: PieceType::Rook,
            capture: None,
        }
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.promote_to == other.promote_to
    }
}

impl ChessState {
    fn gen_pawn_moves(&self, from: u8, moves: &mut Vec<Move>) {
        let forward: i8 = match self.turn {
            PieceColor::White => 8,
            PieceColor::Black => -8,
        };

        let to = (from as i8 + forward) as u8;

        // Advance
        if self.pieces[to as usize].is_none() {
            if to / 8
                == match self.turn {
                    PieceColor::White => 7,
                    PieceColor::Black => 0,
                }
            {
                moves.extend([PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Queen].map(|x| Move {
                    piece_type: PieceType::Pawn,
                    from,
                    to,
                    promote_to: Some(x),
                    ..Default::default()
                }))
            } else {
                moves.push(Move {
                    piece_type: PieceType::Pawn,
                    from,
                    to,
                    ..Default::default()
                });
            }

            if from / 8
                == match self.turn {
                    PieceColor::White => 1,
                    PieceColor::Black => 6,
                }
            {
                let to2 = (to as i8 + forward) as u8;
                if self.pieces[to2 as usize].is_none() {
                    moves.push(Move {
                        piece_type: PieceType::Pawn,
                        from,
                        to: to2,
                        new_en_passant_target: Some(to),
                        ..Default::default()
                    });
                }
            }
        }

        // Capture
        for offset in [1 as i8, -1] {
            if let Some(to) = with_offset(to, offset)
            &&  let Some(p) = self.pieces[to as usize] && p.c != self.turn {
                    if to / 8
                        == match self.turn {
                            PieceColor::White => 7,
                            PieceColor::Black => 0,
                        }
                    {
                        moves.extend(
                            [
                                PieceType::Rook,
                                PieceType::Knight,
                                PieceType::Bishop,
                                PieceType::Queen,
                            ]
                            .map(|x| Move {
                                piece_type: PieceType::Pawn,
                                from,
                                to,
                                capture: Some(p.t),
                                promote_to: Some(x),
                                ..Default::default()
                            }),
                        )
                    } else {
                        moves.push(Move {
                            piece_type: PieceType::Pawn,
                            from,
                            to,
                            capture: Some(p.t),
                            ..Default::default()
                        })
                    }
                }
        }

        // En passant
        if let Some(en_passant_target) = self.en_passant_target {
            for offset in [1 as i8, -1] {
                if with_offset(to, offset) == Some(en_passant_target) {
                    moves.push(Move {
                        piece_type: PieceType::Pawn,
                        from,
                        to: en_passant_target,
                        capture: Some(PieceType::Pawn),
                        en_passant: true,
                        ..Default::default()
                    });
                }
            }
        }
    }

    fn gen_non_pawn_moves(&self, p: Piece, from: u8, moves: &mut Vec<Move>) {
        for offset in p.t.offsets() {
            let mut to = from;
            loop {
                to = match with_offset(to, *offset) {
                    Some(n) => n,
                    None => break,
                };

                if let Some(other) = self.pieces[to as usize] {
                    if other.c != self.turn {
                        moves.push(Move {
                            piece_type: p.t,
                            from,
                            to,
                            capture: Some(other.t),
                            ..Default::default()
                        });
                    }
                    break;
                }

                moves.push(Move {
                    piece_type: p.t,
                    from,
                    to,
                    ..Default::default()
                });

                if !p.t.is_sliding() {
                    break;
                }
            }
        }
    }

    fn gen_castling_moves(&self, moves: &mut Vec<Move>) {
        const CASTLE_OFFSET: PieceColorArray<u8> = PieceColorArray([0, 7 * 8]);
        let offset = CASTLE_OFFSET[self.turn];

        if self.queen_castle[self.turn]
            && !self.check[self.turn]
            && self.pieces[1 + offset as usize].is_none()
            && self.pieces[2 + offset as usize].is_none()
            && self.pieces[3 + offset as usize].is_none()
            && !self.is_square_attacked_by(2 + offset, self.turn.opposite())
            && !self.is_square_attacked_by(3 + offset, self.turn.opposite())
        {
            moves.push(Move {
                piece_type: PieceType::King,
                from: 4 + offset,
                to: 2 + offset,
                castle_queen: true,
                ..Default::default()
            })
        }

        if self.king_castle[self.turn]
            && !self.check[self.turn]
            && self.pieces[6 + offset as usize].is_none()
            && self.pieces[5 + offset as usize].is_none()
            && !self.is_square_attacked_by(6 + offset, self.turn.opposite())
            && !self.is_square_attacked_by(5 + offset, self.turn.opposite())
        {
            moves.push(Move {
                piece_type: PieceType::King,
                castle_king: true,
                from: 4 + offset,
                to: 6 + offset,
                ..Default::default()
            })
        }
    }

    pub fn gen_pseudo_legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::<Move>::new();

        for from in 0u8..64 {
            if let Some(p) = self.pieces[from as usize] {
                if p.c != self.turn {
                    continue;
                }

                match p.t {
                    PieceType::Pawn => self.gen_pawn_moves(from, &mut moves),
                    _ => self.gen_non_pawn_moves(p, from, &mut moves),
                }
            }
        }

        self.gen_castling_moves(&mut moves);
        moves
    }

    pub fn gen_moves(&mut self) -> Vec<Move> {
        let mut moves = self.gen_pseudo_legal_moves();

        // Remove moves that would put the king in check and update whether the move is a check
        moves.retain_mut(|m| {
            self.make_move(m);
            let legal = !self.check[self.turn.opposite()];
            self.unmake_last_move();

            legal
        });

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::chess_engine::{zobrist::Zobrist, ChessState};

    #[test]
    fn startpos_pos_test() {
        let mut state = ChessState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let hash = state.hash;
        let moves = state.gen_moves();
        assert_eq!(moves.len(), 20);
        assert_eq!(state.hash, hash);
        assert_eq!(state.hash, Zobrist::calc_hash(&state));
    }

    #[test]
    fn en_passant_target_test() {
        let mut state = ChessState::from_fen("r3k2r/pppp1ppp/8/8/3Pp3/8/PPP1PPPP/R3K2R b KQkq d3 0 3").unwrap();
        let hash = state.hash;
        let moves = state.gen_moves();
        assert_eq!(moves.len(), 26);
        assert_eq!(state.hash, hash);
        assert_eq!(state.hash, 0xb68cbd4b61a5ece2);
        assert_eq!(state.hash, Zobrist::calc_hash(&state));
    }
}
