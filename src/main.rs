use clap::Parser;
use std::io::{self, Write};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

pub mod sequencer;
use crate::sequencer::Event;
use crate::sequencer::sequencer::{Click, Bar, Song};

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
    let stream_handle = Arc::new(rodio::DeviceSinkBuilder::open_default_sink()?);

    let args = Args::parse();

    let mut clicks = Vec::new();

    for pid in args.pattern.chars() {
        let path = match pid {
            'X' => PathBuf::from("./res/Perc_Can_hi.wav"),
            'x' => PathBuf::from("./res/Perc_Can_lo.wav"),
            _ => PathBuf::from("./res/Perc_Chair_lo.wav"),
        };
        let click =
            Click::init(pid.into(), path, Arc::clone(&stream_handle), tx.clone()).expect("creating click");
        clicks.push(click);
    }

    let bar = Bar {
        tempo: args.tempo,
        unit: args.unit,
        nnotes: args.pattern.len(),
        accents: clicks,
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
