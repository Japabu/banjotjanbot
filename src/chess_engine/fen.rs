use super::{si, zobrist::Zobrist, ChessState, Piece, PieceColor, PieceType, with_offset};

impl ChessState {
    pub fn from_fen(fen: &str) -> Result<ChessState, String> {
        let mut s = ChessState::default();
        let mut f = 0;
        let mut r = 7;
        let splits = fen.split(' ').collect::<Vec<_>>();

        if splits.len() != 6 {
            return Err("Invalid amount of spaces".to_string());
        }

        for c in splits[0].chars() {
            match c {
                'r' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::Black,
                        t: PieceType::Rook,
                    })
                }
                'n' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::Black,
                        t: PieceType::Knight,
                    })
                }
                'b' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::Black,
                        t: PieceType::Bishop,
                    })
                }
                'q' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::Black,
                        t: PieceType::Queen,
                    })
                }
                'k' => {
                    let i = si(f, r);
                    s.pieces[i as usize] = Some(Piece {
                        c: PieceColor::Black,
                        t: PieceType::King,
                    });
                    s.king_pos[PieceColor::Black] = i;
                }
                'p' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::Black,
                        t: PieceType::Pawn,
                    })
                }

                'R' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::White,
                        t: PieceType::Rook,
                    })
                }
                'N' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::White,
                        t: PieceType::Knight,
                    })
                }
                'B' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::White,
                        t: PieceType::Bishop,
                    })
                }
                'Q' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::White,
                        t: PieceType::Queen,
                    })
                }
                'K' => {
                    let i = si(f, r);
                    s.pieces[i as usize] = Some(Piece {
                        c: PieceColor::White,
                        t: PieceType::King,
                    });
                    s.king_pos[PieceColor::White] = i;
                }
                'P' => {
                    s.pieces[si(f, r) as usize] = Some(Piece {
                        c: PieceColor::White,
                        t: PieceType::Pawn,
                    })
                }

                '/' => {
                    r -= 1;
                    f = 0;
                    continue;
                }

                '1'..='8' => {
                    f += c as u8 - b'1' + 1;
                    continue;
                }

                _ => return Err(format!("Invalid character {} in board", c)),
            }
            f += 1;
        }

        s.turn = match splits[1] {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => return Err("Invalid active color".to_string()),
        };

        for c in splits[2].chars() {
            match c {
                '-' => break,
                'K' => s.king_castle[PieceColor::White] = true,
                'Q' => s.queen_castle[PieceColor::White] = true,
                'k' => s.king_castle[PieceColor::Black] = true,
                'q' => s.queen_castle[PieceColor::Black] = true,
                _ => return Err(format!("Invalid character {} in castling rights", c)),
            }
        }

        let new_en_passant_target = match splits[3].as_bytes() {
            [b'-'] => None,
            [file @ b'a'..=b'h', rank @ b'1'..=b'8'] => Some(si(file - b'a', rank - b'1') as u8),
            _ => return Err("Invalid en passant target".to_string()),
        };

        // Check if en passant is possible
        if let Some(sq) = new_en_passant_target {
            let sq = match s.turn {
                PieceColor::White => sq - 8,
                PieceColor::Black => sq + 8,
            };

            for i in [-1, 1].iter().filter_map(|o| with_offset(sq, *o)) {
                match s.pieces[i as usize] {
                    Some(Piece {
                        c,
                        t: PieceType::Pawn,
                    }) if c == s.turn => {
                        s.en_passant_target = new_en_passant_target;
                        break;
                    }
                    _ => (),
                }
            }
        }

        s.halfmove_clock = match splits[4].parse::<u8>() {
            Ok(x) => x,
            Err(_) => return Err("Invalid halfmove clock".to_string()),
        };

        s.move_clock = match splits[5].parse::<u8>() {
            Ok(x) => x,
            Err(_) => return Err("Invalid move clock".to_string()),
        };

        s.update_check();

        s.hash = Zobrist::calc_hash(&s);

        Ok(s)
    }
}
