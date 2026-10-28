use crate::sequencer::Event;
use std::io::{self, Write};
use std::sync::mpsc;

pub fn run(rx: mpsc::Receiver<Event>, _nnotes: i32) {
    for event in rx {
        match event {
            Event::Click(click) => print!("{}", click),
            Event::Bar(_) => print!("|"),
        }
        let _ = io::stdout().flush();
    }
}
