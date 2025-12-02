# libchess
A chess library for making chess engines and GUIs written in Rust

contains features such as position representation, bitboards, move generation, and other utility stuff

move gen is decently quick at about ~15-18m nodes/s on a 10 year old cpu

---

# Quick Start

before using most functions, you'll need to initate the library
```rs
let (masks, zb) = libchess::init();
```
this gives you two variables you'll use often as parameters to functions such as move generation, *masks* is a collection of attack masks for different pieces, while *zb* is a collection of values used for generating position keys

to create a position you can use the following:
```rs
let mut pos = pos::Position::from_fen(pos::START_FEN, &zb); // create a position from a FEN string, the library provides the FEN for the starting position, but you can use your own
let mut pos = pos::Position::blank(); // create a blank position, containing no pieces
```

to generate a list of legal moves for a position:
```rs
let list = moves::gen_legal(&mut pos, &masks, &zb);
```
---
GUI made with libchess: [chess_tail](https://github.com/irrelevantpiadina/chess_tail)
