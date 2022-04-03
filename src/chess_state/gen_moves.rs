use std::default;

use crate::chess_state::PieceColorArray;

use super::{ChessState, Piece, PieceColor, PieceType};

const MAILBOX: [Option<usize>; 120] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(0),
    Some(1),
    Some(2),
    Some(3),
    Some(4),
    Some(5),
    Some(6),
    Some(7),
    None,
    None,
    Some(8),
    Some(9),
    Some(10),
    Some(11),
    Some(12),
    Some(13),
    Some(14),
    Some(15),
    None,
    None,
    Some(16),
    Some(17),
    Some(18),
    Some(19),
    Some(20),
    Some(21),
    Some(22),
    Some(23),
    None,
    None,
    Some(24),
    Some(25),
    Some(26),
    Some(27),
    Some(28),
    Some(29),
    Some(30),
    Some(31),
    None,
    None,
    Some(32),
    Some(33),
    Some(34),
    Some(35),
    Some(36),
    Some(37),
    Some(38),
    Some(39),
    None,
    None,
    Some(40),
    Some(41),
    Some(42),
    Some(43),
    Some(44),
    Some(45),
    Some(46),
    Some(47),
    None,
    None,
    Some(48),
    Some(49),
    Some(50),
    Some(51),
    Some(52),
    Some(53),
    Some(54),
    Some(55),
    None,
    None,
    Some(56),
    Some(57),
    Some(58),
    Some(59),
    Some(60),
    Some(61),
    Some(62),
    Some(63),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

const MAILBOX64: [usize; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28, 31, 32, 33, 34, 35, 36, 37, 38, 41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58, 61, 62, 63, 64, 65, 66, 67, 68, 71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88, 91, 92, 93, 94, 95, 96, 97, 98,
];

pub const fn with_offset(from: usize, offset: i8) -> Option<usize> {
    MAILBOX[(MAILBOX64[from] as i8 + offset) as usize]
}

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

#[derive(Clone, Copy, Eq)]
pub struct Move {
    pub pt: PieceType,
    pub from: usize,
    pub to: usize,
    pub promote_to: Option<PieceType>,
    pub new_en_passant_target: Option<usize>,
    pub castle_king: bool,
    pub castle_queen: bool,
    pub en_passant: bool,
    pub check: bool,
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
            check: false,
            pt: PieceType::Rook,
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
    fn gen_pawn_moves(&self, from: usize, moves: &mut Vec<Move>) {
        let forward: i8 = match self.turn {
            PieceColor::White => 8,
            PieceColor::Black => -8,
        };

        let to = (from as i8 + forward) as usize;

        // Advance
        if self.pieces[to].is_none() {
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
                        pt: PieceType::Pawn,
                        from,
                        to,
                        promote_to: Some(x),
                        ..Default::default()
                    }),
                )
            } else {
                moves.push(Move {
                    pt: PieceType::Pawn,
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
                let to2 = (to as i8 + forward) as usize;
                if self.pieces[to2].is_none() {
                    moves.push(Move {
                        pt: PieceType::Pawn,
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
            &&  let Some(p) = self.pieces[to] && p.c != self.turn {
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
                                pt: PieceType::Pawn,
                                from,
                                to,
                                capture: Some(p.t),
                                promote_to: Some(x),
                                ..Default::default()
                            }),
                        )
                    } else {
                        moves.push(Move {
                            pt: PieceType::Pawn,
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
                        pt: PieceType::Pawn,
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

    fn gen_non_pawn_moves(&self, p: Piece, from: usize, moves: &mut Vec<Move>) {
        for offset in p.t.offsets() {
            let mut to = from;
            loop {
                to = match with_offset(to, *offset) {
                    Some(n) => n,
                    None => break,
                };

                if let Some(other) = self.pieces[to] {
                    if other.c != self.turn {
                        moves.push(Move {
                            pt: p.t,
                            from,
                            to,
                            capture: Some(other.t),
                            ..Default::default()
                        });
                    }
                    break;
                }

                moves.push(Move {
                    pt: p.t,
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

    pub fn is_square_attacked(&self, attacker: PieceColor, square: usize) -> bool {
        // Check if square is attacked orthogonally by a rook, queen or king
        for offset in PieceType::Rook.offsets() {
            let mut slid = false;
            let mut to = square;
            loop {
                to = match with_offset(to, *offset) {
                    Some(n) => n,
                    None => break,
                };

                match self.pieces[to] {
                    Some(Piece {
                        c,
                        t: PieceType::Rook | PieceType::Queen,
                    }) if c == attacker => return true,
                    Some(Piece {
                        c,
                        t: PieceType::King,
                    }) if c == attacker && !slid => return true,
                    Some(_) => break,
                    _ => (),
                }

                slid = true;
            }
        }

        // Check if square is diagonally attacked by a bishop, queen or king
        for offset in PieceType::Bishop.offsets() {
            let mut slid = false;
            let mut to = square;
            loop {
                to = match with_offset(to, *offset) {
                    Some(n) => n,
                    None => break,
                };

                match self.pieces[to] {
                    Some(Piece {
                        c,
                        t: PieceType::Bishop | PieceType::Queen,
                    }) if c == attacker => return true,
                    Some(Piece {
                        c,
                        t: PieceType::King,
                    }) if c == attacker && !slid => return true,
                    Some(_) => break,
                    _ => (),
                }

                slid = true;
            }
        }

        // Check if square is attacked by a knight
        for offset in PieceType::Knight.offsets() {
            let to = match with_offset(square, *offset) {
                Some(n) => n,
                None => continue,
            };

            match self.pieces[to] {
                Some(Piece {
                    c,
                    t: PieceType::Knight,
                }) if c == attacker => return true,
                _ => (),
            }
        }

        // Check if square is attacked by a pawn
        let backward = match attacker {
            PieceColor::White => -10,
            PieceColor::Black => 10,
        };

        for offset in [1 as i8, -1] {
            let to = match with_offset(square, backward + offset) {
                Some(n) => n,
                None => continue,
            };

            match self.pieces[to] {
                Some(Piece {
                    c,
                    t: PieceType::Pawn,
                }) if c == attacker => return true,
                _ => (),
            }
        }

        false
    }

    fn gen_castling_moves(&self, moves: &mut Vec<Move>) {
        const CASTLE_OFFSET: PieceColorArray<usize> = PieceColorArray([0, 7 * 8]);
        let offset = CASTLE_OFFSET[self.turn];

        if self.queen_castle[self.turn]
            && !self.check
            && self.pieces[1 + offset].is_none()
            && self.pieces[2 + offset].is_none()
            && self.pieces[3 + offset].is_none()
            && !self.is_square_attacked(self.turn.oppo(), 2 + offset)
            && !self.is_square_attacked(self.turn.oppo(), 3 + offset)
        {
            moves.push(Move {
                pt: PieceType::King,
                from: 4 + offset,
                to: 2 + offset,
                castle_queen: true,
                ..Default::default()
            })
        }

        if self.king_castle[self.turn]
            && !self.check
            && self.pieces[6 + offset].is_none()
            && self.pieces[5 + offset].is_none()
            && !self.is_square_attacked(self.turn.oppo(), 6 + offset)
            && !self.is_square_attacked(self.turn.oppo(), 5 + offset)
        {
            moves.push(Move {
                pt: PieceType::King,
                castle_king: true,
                from: 4 + offset,
                to: 6 + offset,
                ..Default::default()
            })
        }
    }

    pub fn gen_moves(&self) -> Vec<Move> {
        let mut moves = Vec::<Move>::new();

        for from in 0..64 {
            if let Some(p) = self.pieces[from] {
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

        // Remove moves that would put the king in check and update whether the move is a check
        moves.retain_mut(|m| {
            let mut s = *self;
            s.make_move(m);

            if s.is_square_attacked(s.turn, s.king_pos[self.turn]) {
                return false;
            }

            m.check = s.is_square_attacked(self.turn, s.king_pos[s.turn]);

            true
        });

        moves
    }
}
