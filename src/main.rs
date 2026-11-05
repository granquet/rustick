use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc;

mod gui;
pub mod sequencer;
pub mod terminal;
use crate::sequencer::{Bar, ClickKind, Clicks, Sequencer, Song};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    bars: Option<u32>,
    #[arg(short, long, default_value_t = 80)]
    tempo: u32,
    #[arg(short, long, default_value_t = 4)]
    unit: u32,
    #[arg(short, long, default_value_t = "Xxxx".into())]
    pattern: String,
    #[arg(long, default_value_t = false)]
    gui: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    let args = Args::parse();

    let mut clicks = Vec::new();

    for pid in args.pattern.chars() {
        clicks.push(ClickKind::from(pid));
    }

    let bar = Bar {
        tempo: args.tempo,
        unit: args.unit,
        nnotes: args.pattern.len(),
        accents: clicks,
    };

    let song = Song {
        bars: vec![bar],
        loops: args.bars,
        clicks: Clicks::init(
            PathBuf::from("./res/Perc_Can_hi.wav"),
            PathBuf::from("./res/Perc_Can_lo.wav"),
            PathBuf::from("./res/Perc_Chair_lo.wav"),
        )?,
    };

    let sequencer = Sequencer::new(song, tx.clone());

    std::thread::spawn(move || {
        sequencer.run();
    });

    if args.gui {
        gui::run(rx, args.pattern.len() as i32);
    } else {
        terminal::run(rx, args.pattern.len() as i32);
    }

    Ok(())
}
