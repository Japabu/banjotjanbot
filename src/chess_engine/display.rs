use std::fmt::{Display};

use super::{gen_moves::Move, si, ChessState, Piece, PieceColor, PieceType};

fn piece_at(s: &ChessState, f: u8, r: u8) -> String {
    s.pieces[si(f, r) as usize]
        .as_ref()
        .map_or("+".to_string(), |x| format!("{}", x))
}

fn format_square(i: u8) -> String {
    let (fi, ri) = super::fr(i);

    let f = (b'a' + fi) as char;
    let r = (b'1' + ri) as char;
    format!("{}{}", f, r)
}

impl Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PieceType::Rook => "r",
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Queen => "q",
                PieceType::King => "k",
                PieceType::Pawn => "p",
            }
        )?;

        Ok(())
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if self.c == PieceColor::White {
                self.t.to_string().to_uppercase()
            } else {
                self.t.to_string()
            }
        )?;

        Ok(())
    }
}

impl Display for ChessState {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in (0..8).rev() {
            for f in 0..8 {
                let c = piece_at(self, f, r);
                write!(fmt, "{}", c)?;
            }
            writeln!(fmt)?;
        }

        writeln!(fmt, "Active color: {:?}", self.turn)?;
        if self.king_castle[PieceColor::White] {
            writeln!(fmt, "White can castle short")?;
        }
        if self.queen_castle[PieceColor::White] {
            writeln!(fmt, "White can castle long")?;
        }
        if self.king_castle[PieceColor::Black] {
            writeln!(fmt, "Black can castle short")?;
        }
        if self.queen_castle[PieceColor::Black] {
            writeln!(fmt, "Black can castle long")?;
        }
        if let Some(x) = self.en_passant_target {
            writeln!(fmt, "En passant target: {:?}", x)?;
        }
        writeln!(fmt, "Halfmove count: {}", self.halfmove_clock)?;
        writeln!(fmt, "Move count: {}", self.move_clock)?;

        writeln!(
            fmt,
            "White king pos: {}",
            format_square(self.king_pos[PieceColor::White])
        )?;
        write!(
            fmt,
            "Black king pos: {}",
            format_square(self.king_pos[PieceColor::Black])
        )?;

        Ok(())
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", format_square(self.from), format_square(self.to))?;

        if let Some(t) = self.promote_to {
            write!(f, "{}", t)?;
        }

        Ok(())
    }
}
