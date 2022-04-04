use std::{io::stdin, time::Duration};

use chessai::chess_state::{
    gen_moves::Move, transposition_table::TranspositionTable, zobrist::Zobrist, ChessState,
};

fn perft(state: ChessState, m: Option<&Move>, depth: u32) -> [u64; 6] {
    if depth == 0 {
        if let Some(m) = m {
            return [
                1,
                m.capture.is_some() as u64,
                m.en_passant as u64,
                (m.castle_king || m.castle_queen) as u64,
                m.promote_to.is_some() as u64,
                m.check as u64,
            ];
        } else {
            return [1, 0, 0, 0, 0, 0];
        }
    }

    state
        .gen_moves()
        .iter()
        .map(|m| {
            let mut s = state;
            s.make_move(m);
            perft(s, Some(m), depth - 1)
        })
        .fold([0; 6], |mut a, x| {
            for i in 0..a.len() {
                a[i] += x[i];
            }
            a
        })
}

fn fmt_moves(moves: &[Move]) -> String {
    let mut s = String::new();
    for m in moves {
        s.push_str(&format!("{} ", m));
    }
    s
}

fn main() {
    TranspositionTable::init();

    println!("Stupid chess engine by Jan");

    let mut chess_state = ChessState::default();
    loop {
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();

        let mut splits = line.trim_end().split(" ");
        let cmd = match splits.next() {
            Some(cmd) => cmd,
            None => continue,
        };

        let args = splits.collect::<Vec<_>>();
        match cmd {
            "position" => {
                if args.len() < 1 {
                    println!("Invalid amount of arguments!");
                    continue;
                }

                chess_state = match args[0] {
                    "startpos" => ChessState::from_fen(
                        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                    )
                    .unwrap(),
                    "fen" => {
                        if args.len() < 2 {
                            println!("Missing fen!");
                            continue;
                        }

                        let fen = args[1..].join(" ");

                        match ChessState::from_fen(fen.as_str()) {
                            Ok(p) => p,
                            Err(err) => {
                                println!("{}", err);
                                continue;
                            }
                        }
                    }
                    _ => {
                        println!("Invalid argument!");
                        continue;
                    }
                }
            }
            "d" => {
                println!("{}", chess_state);
            }
            "m" => {
                let moves = chess_state.gen_moves();

                if args.len() == 1 {
                    let i: usize = args[0].parse().unwrap();
                    chess_state.make_move(&moves[i - 1]);
                } else {
                    for (i, m) in moves.iter().enumerate() {
                        println!("{}: {}", i + 1, m);
                    }
                }
            }
            "perft" => {
                if args.len() != 1 {
                    println!("Invalid amount of arguments!");
                    continue;
                }

                let depth: u32 = match args[0].parse() {
                    Ok(depth) => depth,
                    _ => {
                        println!("Invalid depth!");
                        continue;
                    }
                };

                let mut a = [0; 6];
                let moves = chess_state.gen_moves();
                for m in moves {
                    let mut s = chess_state;
                    s.make_move(&m);

                    let p = perft(s, None, depth - 1);
                    println!("{}{:?}", m, p);

                    for i in 0..a.len() {
                        a[i] += p[i];
                    }
                }
                println!("Total: {:?}", a);
            }
            "go" => {
                if args.len() != 1 {
                    println!("Invalid amount of arguments!");
                    continue;
                }

                let depth: u8 = match args[0].parse() {
                    Ok(depth) => depth,
                    _ => {
                        println!("Invalid depth!");
                        continue;
                    }
                };

                let (eval, moves) = chess_state.eval(Some(depth), None);
                println!("{} {}", eval, fmt_moves(&moves));
                println!("bestmove {}", moves[0]);
            }
            "gotime" => {
                if args.len() != 1 {
                    println!("Invalid amount of arguments!");
                    continue;
                }

                let seconds: u64 = match args[0].parse() {
                    Ok(time) => time,
                    _ => {
                        println!("Invalid time!");
                        continue;
                    }
                };

                let (eval, moves) = chess_state.eval(None, Some(Duration::from_secs(seconds)));
                println!("{} {}", eval, fmt_moves(&moves));
                println!("bestmove {}", moves[0]);
            }
            _ => println!("Unknown command!"),
        }
    }
}
