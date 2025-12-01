use std::fs;

use colored::Colorize;

use crate::{AttackMasks, ZobristValues, moves, pos};

/// a standard perft test
///
/// recursively searches a position with a certain depth, useful for testing the correctness of move generation
pub fn perft(
    pos: &mut pos::Position,
    depth: i32,
    is_root: bool,
    masks: &AttackMasks,
    zb: &ZobristValues,
) -> i64 {
    let mut nodes = 0;

    if depth == 0 {
        return 1;
    }

    for mov in moves::gen_legal(pos, masks, zb) {
        pos.make_move(mov, zb);
        let new_nodes = perft(pos, depth - 1, false, masks, zb);
        nodes += new_nodes;
        if is_root {
            println!("{}: {new_nodes}", mov.to_uci_fmt())
        }
        pos.unmake_move();
    }

    if is_root {
        println!("\nsearched {nodes} nodes");
    }

    nodes
}

/// parses an epd file containing perft test positions and compares the results in the file
/// to the results given by the perft function
pub fn test_epd(
    path: &str,
    max_depth: i32,
    num_tests: i32,
    start_at: usize,
    masks: &AttackMasks,
    zb: &ZobristValues,
) {
    #[derive(Debug)]
    struct TestCase<'a> {
        fen: &'a str,
        depths: Vec<i32>,
        node_counts: Vec<i64>,
    }

    let test_cases =
        String::from_utf8_lossy(&fs::read(path).expect("failed to read file")).to_string();

    let lines: Vec<&str> = test_cases.split('\n').collect();

    let mut test_cases: Vec<TestCase> = Vec::new();

    for line in lines {
        let fen = line.split(';').nth(0).unwrap();
        let node_counts: Vec<i64> = line
            .replace(" ", "")
            .split(";D")
            .skip(1)
            .map(|n| {
                n.chars()
                    .skip(1)
                    .collect::<String>()
                    .trim()
                    .parse()
                    .unwrap()
            })
            .collect();

        let depths: Vec<i32> = line
            .replace(" ", "")
            .split(";D")
            .skip(1)
            .map(|n| {
                n.chars()
                    .take(1)
                    .collect::<String>()
                    .trim()
                    .parse()
                    .unwrap()
            })
            .collect();

        test_cases.push(TestCase {
            fen,
            depths,
            node_counts,
        });
    }

    let mut ok = 0;
    let mut failed = 0;
    let mut i = 0;

    for test_case in test_cases.iter().skip(start_at) {
        if test_case.depths[0] > max_depth {
            continue;
        }
        println!("\ntesting position: {}", test_case.fen.bright_yellow());

        let mut j = 0;
        for &node_count in &test_case.node_counts {
            if test_case.depths[j] > max_depth {
                break;
            }
            print!(
                "depth: {}; expected nodes: {}; ",
                test_case.depths[j],
                node_count.to_string().yellow()
            );
            let nodes = perft(
                &mut pos::Position::from_fen(test_case.fen, zb),
                test_case.depths[j],
                false,
                masks,
                zb,
            );
            if nodes == node_count {
                println!(
                    "actual nodes: {}; {}",
                    nodes.to_string().yellow(),
                    "ok".green()
                );
                ok += 1;
            } else {
                println!(
                    "actual nodes: {} ({}); {}",
                    nodes.to_string().red(),
                    {
                        let s = String::from(if nodes > node_count { "+" } else { "" });
                        (s + (nodes - node_count).to_string().as_str()).red()
                    },
                    "failed".red()
                );
                failed += 1;
            }

            j += 1;
        }

        i += 1;

        if i == num_tests {
            break;
        }
    }

    let all = ok + failed;

    println!(
        "results: out of {} tests, {} passed, {} failed",
        all.to_string().yellow().bold(),
        if ok == all {
            ok.to_string().green().bold()
        } else {
            ok.to_string().yellow().bold()
        },
        if failed == all {
            failed.to_string().red().bold()
        } else if failed == 0 {
            failed.to_string().green().bold()
        } else {
            failed.to_string().red().bold()
        }
    );
}
