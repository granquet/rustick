use clap::Parser;
use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc;

pub mod sequencer;
use crate::sequencer::Event;
use crate::sequencer::{Bar, ClickKind, Clicks, Song};

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
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    let args = Args::parse();

    let mut clicks = Vec::new();

    for pid in args.pattern.chars() {
        let click = match pid {
            'X' => ClickKind::Accent,
            'x' => ClickKind::Normal,
            '-' => ClickKind::Upbeat,
            _ => ClickKind::Rest,
        };
        clicks.push(click);
    }

    let bar = Bar {
        tempo: args.tempo,
        unit: args.unit,
        nnotes: args.pattern.len(),
        accents: clicks,
        clicks: Clicks::init(
            PathBuf::from("./res/Perc_Can_hi.wav"),
            PathBuf::from("./res/Perc_Can_lo.wav"),
            PathBuf::from("./res/Perc_Chair_lo.wav"),
            tx.clone(),
        )?,
        tx: tx.clone(),
    };

    let song = Song {
        bars: vec![bar],
        loops: args.bars,
        tx: tx.clone(),
    };

    std::thread::spawn(move || {
        song.play();
    });

    for event in rx {
        match event {
            Event::Click(click) => print!("{}", click),
            Event::Bar(_) => print!("|"),
        }
        let _ = io::stdout().flush();
    }

    Ok(())
}
