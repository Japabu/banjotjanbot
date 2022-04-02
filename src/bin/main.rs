use std::{env, io::stdin};

use chessai::chess_state::{gen_moves::Move, ChessState};

fn perft(state: ChessState, m: Option<&Move>, depth: u32) -> [u64; 6] {
    if depth == 0 {
        if let Some(m) = m {
            return [
                1,
                m.capture as u64,
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

fn main() {
    if env::args().len() == 3 {
        let fen = &env::args().nth(1).unwrap();
        let depth = env::args().nth(2).unwrap().parse::<u32>().unwrap();

        let chess_state = ChessState::from_fen(fen).unwrap();
        print!(
            "{}",
            chess_state.find_best_move_with_depth(depth).0.unwrap()
        );
        return;
    }

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

                let depth: u32 = match args[0].parse() {
                    Ok(depth) => depth,
                    _ => {
                        println!("Invalid depth!");
                        continue;
                    }
                };

                let (m, eval) = chess_state.find_best_move_with_depth(depth);

                println!(
                    "{} {}",
                    m.map_or_else(|| String::from("None"), |x| x.to_string()),
                    eval
                );
            }
            "gotime" => {
                if args.len() != 1 {
                    println!("Invalid amount of arguments!");
                    continue;
                }

                let time: u64 = match args[0].parse() {
                    Ok(time) => time,
                    _ => {
                        println!("Invalid time!");
                        continue;
                    }
                };

                let (m, eval, depth) = chess_state.find_best_move_with_time(time);

                println!(
                    "{} {} {}",
                    m.map_or_else(|| String::from("None"), |x| x.to_string()),
                    eval,
                    depth,
                );
            }
            "eval" => {
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

                println!("{}", chess_state.absolute_eval(depth));
            }
            _ => println!("Unknown command!"),
        }
    }
}
