use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind},
    sync::RwLock,
};

use byteorder::{BigEndian, ReadBytesExt};

use super::{gen_moves::Move, PieceType};

static mut BOOK: Option<RwLock<Book>> = None;

struct BookEntry {
    key: u64,
    move_: String,
}

impl BookEntry {
    fn new(key: u64, move_: u16) -> Self {
        let to_file = (move_ & 7) as u8;
        let to_row = ((move_ >> 3) & 7) as u8;
        let from_file = ((move_ >> 6) & 7) as u8;
        let from_row = ((move_ >> 9) & 7) as u8;
        let promote_to = match (move_ >> 12) & 7 {
            0 => None,
            1 => Some(PieceType::Knight),
            2 => Some(PieceType::Bishop),
            3 => Some(PieceType::Rook),
            4 => Some(PieceType::Queen),
            _ => panic!("Invalid piece"),
        };

        let m = Move {
            from: from_row * 8 + from_file,
            to: to_row * 8 + to_file,
            promote_to,
            ..Default::default()
        };

        Self {
            key,
            move_: m.to_string(),
        }
    }
}

pub struct Book {
    entries: Vec<BookEntry>,
}

impl Book {
    pub fn load(path: &str) -> Result<(), Error> {
        let mut entries: Vec<BookEntry> = Vec::new();

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        loop {
            let key = match reader.read_u64::<BigEndian>() {
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => panic!("{}", e),
                Ok(key) => key,
            };

            let move_ = reader.read_u16::<BigEndian>()?;

            entries.push(BookEntry::new(key, move_));

            reader.seek_relative(6)?;
        }

        entries.sort_by_key(|e| e.key);

        unsafe {
            BOOK = Some(RwLock::new(Book { entries }));
        }

        Ok(())
    }

    pub fn get(key: u64) -> Option<String> {
        let book = unsafe { BOOK.as_ref() }.unwrap().read().unwrap();

        book.entries
            .binary_search_by_key(&key, |e| e.key)
            .ok()
            .map(|i| book.entries[i].move_.clone())
    }
}
