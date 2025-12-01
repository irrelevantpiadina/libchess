#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::time::Instant;

use libchess::{moves, perft, piece::bb::BitboardUtil, pos};

#[test]
fn test_main() {
    let (masks, zb) = libchess::init();
    let mut pos = pos::Position::from_fen(pos::START_FEN, &zb);

    let timer = Instant::now();
    perft::perft(&mut pos, 6, true, &masks, &zb);
    // perft::test_epd("perftsuite.epd", 6, 200, 0, &masks, &zb);
    println!("perft took {}s", timer.elapsed().as_secs_f32());
}
