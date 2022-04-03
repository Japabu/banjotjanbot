use std::sync::RwLock;

use super::{gen_moves::Move, ChessState};

const TRANSPOSITION_ENTRIES: usize = 2 << 26;

static mut TRANSPOSITION_TABLE: Option<RwLock<TranspositionTable>> = None;

pub struct TranspositionTable {
    entries: Vec<Option<TranspositionEntry>>,
}

#[derive(Clone, Copy)]
pub struct TranspositionEntry {
    pub depth: u32,
    pub score: i32,
    pub best_move: Option<Move>,
}

impl TranspositionTable {
    pub fn init() {
        unsafe {
            TRANSPOSITION_TABLE = Some(RwLock::new(TranspositionTable {
                entries: vec![None; TRANSPOSITION_ENTRIES],
            }));
        }
    }

    pub fn get(state: &ChessState) -> Option<TranspositionEntry> {
        let transposition_table = unsafe { TRANSPOSITION_TABLE.as_ref() }
            .unwrap()
            .read()
            .unwrap();

        transposition_table.entries[state.hash as usize % TRANSPOSITION_ENTRIES]
    }

    pub fn set(state: &ChessState, entry: TranspositionEntry) {
        let mut transposition_table = unsafe { TRANSPOSITION_TABLE.as_ref() }
            .unwrap()
            .write()
            .unwrap();

        transposition_table.entries[state.hash as usize % TRANSPOSITION_ENTRIES] = Some(entry);
    }
}
