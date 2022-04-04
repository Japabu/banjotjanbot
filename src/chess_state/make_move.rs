use super::{gen_moves::Move, ChessState, Piece, PieceColor, PieceColorArray, PieceType};

const CASTLE_OFFSET: PieceColorArray<usize> = PieceColorArray([0, 7 * 8]);
const QUEEN_CASTLE_SQUARES: PieceColorArray<[usize; 2]> =
    PieceColorArray([[0, 4], [0 + 7 * 8, 4 + 7 * 8]]);
const KING_CASTLE_SQUARES: PieceColorArray<[usize; 2]> =
    PieceColorArray([[4, 7], [4 + 7 * 8, 7 + 7 * 8]]);

impl ChessState {
    pub fn make_move(&mut self, m: &Move) {
        self.inc_update(m);

        self.en_passant_target = m.new_en_passant_target;
        self.check = m.check;

        if m.pt == PieceType::Pawn || m.capture.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.turn == PieceColor::White {
            self.move_clock += 1;
        }

        if m.castle_queen {
            let offset = CASTLE_OFFSET[self.turn];
            self.pieces[2 + offset] = self.pieces[4 + offset];
            self.pieces[3 + offset] = self.pieces[0 + offset];
            self.pieces[0 + offset] = None;
            self.pieces[4 + offset] = None;
            self.queen_castle[self.turn] = false;
            self.king_castle[self.turn] = false;
            self.king_pos[self.turn] = 2 + offset;
            self.turn = self.turn.oppo();
            return;
        } else if m.castle_king {
            let offset = CASTLE_OFFSET[self.turn];
            self.pieces[6 + offset] = self.pieces[4 + offset];
            self.pieces[5 + offset] = self.pieces[7 + offset];
            self.pieces[7 + offset] = None;
            self.pieces[4 + offset] = None;
            self.queen_castle[self.turn] = false;
            self.king_castle[self.turn] = false;
            self.king_pos[self.turn] = 6 + offset;
            self.turn = self.turn.oppo();
            return;
        }

        for color in [PieceColor::White, PieceColor::Black] {
            self.queen_castle[color] &= !QUEEN_CASTLE_SQUARES[color].contains(&m.from)
                && !QUEEN_CASTLE_SQUARES[color].contains(&m.to);

            self.king_castle[color] &= !KING_CASTLE_SQUARES[color].contains(&m.from)
                && !KING_CASTLE_SQUARES[color].contains(&m.to);
        }

        if m.pt == PieceType::King {
            self.king_pos[self.turn] = m.to;
        }

        self.pieces[m.to] = self.pieces[m.from];
        self.pieces[m.from] = None;

        if m.en_passant {
            self.pieces[(m.to as i8
                + match self.turn {
                    PieceColor::White => -8,
                    PieceColor::Black => 8,
                }) as usize] = None;
        }

        if let Some(t) = m.promote_to {
            self.pieces[m.to] = Some(Piece { c: self.turn, t });
        }

        self.turn = self.turn.oppo();
    }
}
