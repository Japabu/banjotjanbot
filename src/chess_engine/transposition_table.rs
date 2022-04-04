use std::sync::RwLock;

use super::gen_moves::Move;

const TRANSPOSITION_ENTRIES: usize = 2 << 25;

static mut TRANSPOSITION_TABLE: Option<RwLock<TranspositionTable>> = None;

pub struct TranspositionTable {
    entries: Vec<Option<TranspositionEntry>>,
}

#[derive(Clone, Copy)]
pub struct TranspositionEntry {
    pub key: u64,
    pub depth: u8,
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

    pub fn get(key: u64) -> Option<TranspositionEntry> {
        let transposition_table = unsafe { TRANSPOSITION_TABLE.as_ref() }
            .unwrap()
            .read()
            .unwrap();

        if let Some(entry) = transposition_table.entries[key as usize % TRANSPOSITION_ENTRIES] {
            if entry.key == key {
                return Some(entry);
            } else {
                panic!("Collision detected!");
            }
        }
        None
    }

    pub fn set(key: u64, entry: TranspositionEntry) {
        let mut transposition_table = unsafe { TRANSPOSITION_TABLE.as_ref() }
            .unwrap()
            .write()
            .unwrap();

        transposition_table.entries[key as usize % TRANSPOSITION_ENTRIES] = Some(entry);
    }
}
