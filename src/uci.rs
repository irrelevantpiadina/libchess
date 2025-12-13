use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdout, Command, Stdio},
};

use crate::{moves, pos};

pub const UCI: &str = "uci";
pub const UCI_OK: &str = "uciok";
pub const NEW_GAME: &str = "ucinewgame";
pub const IS_READY: &str = "isready";
pub const READY_OK: &str = "readyok";
pub const BEST_MOVE: &str = "bestmove";
pub const STOP: &str = "stop";

/// struct for communicating with UCI engines from a gui
pub struct Engine {
    exe: Child,
    stdout_reader: BufReader<ChildStdout>,
    buf: String,
}

impl Engine {
    pub fn new(path: &str) -> io::Result<Self> {
        let mut exe = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout_reader = BufReader::new(exe.stdout.take().unwrap());

        Ok(Engine {
            exe,
            stdout_reader,
            buf: String::new(),
        })
    }

    /// send a command to an engine
    #[inline(always)]
    pub fn send(&mut self, cmd: &str) -> io::Result<()> {
        self.exe
            .stdin
            .as_mut()
            .unwrap()
            .write(format!("{cmd}\n").as_bytes())?;

        Ok(())
    }

    /// returns the next line of output from an engine
    #[inline(always)]
    pub fn get_next(&mut self) -> io::Result<&str> {
        self.buf.clear();

        self.stdout_reader.read_line(&mut self.buf)?;

        Ok(&self.buf)
    }

    /// if the next line of output contains `expected`,
    /// the function returns the entire line,
    /// otherwise `None`
    #[inline(always)]
    pub fn try_get(&mut self, expected: &str) -> Option<&str> {
        if self.get_next().ok()?.contains(expected) {
            Some(&self.buf)
        } else {
            None
        }
    }

    /// asks the engine to make a move
    pub fn request_move(
        &mut self,
        pos: &pos::Position,
        starting_fen: &str,
        wtime_ms: u128,
        btime_ms: u128,
    ) -> io::Result<()> {
        let moves = pos
            .history()
            .iter()
            .filter(|st| st.move_played.is_some())
            .map(|st| st.move_played.unwrap().to_uci_fmt() + " ")
            .collect::<String>();

        let moves = match pos.move_played() {
            Some(mov) => String::from("moves ") + &moves + &mov.to_uci_fmt(),
            None => moves,
        };

        if starting_fen == pos::START_FEN {
            self.send(&format!("position startpos {moves}"))?;
        } else {
            self.send(&format!("position fen {starting_fen} {moves}"))?;
        }
        self.send(&format!("go wtime {wtime_ms} btime {btime_ms}"))?;

        Ok(())
    }

    /// returns the move an engine wants to play after being prompted by `Engine::request_move()`
    ///
    /// if the engine returns a null move, the function returns `Some(None)`, if no move is received, `None` is returned, else `Some(Some(Move))`
    #[inline(always)]
    pub fn try_get_move(&mut self, pos: &pos::Position) -> Option<Option<moves::Move>> {
        match self.try_get(BEST_MOVE) {
            Some(mov) => {
                if mov.contains("none") || mov.contains("0000") {
                    Some(None)
                } else {
                    Some(Some(moves::Move::from_str_move(
                        mov.split(" ").nth(1).unwrap(),
                        pos,
                    )))
                }
            }
            None => None,
        }
    }
}
