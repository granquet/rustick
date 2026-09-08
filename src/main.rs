use clap::Parser;
use rodio::{Decoder, MixerDeviceSink, Source, source::Buffered};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
struct Click {
    name: String,
    buffer: Buffered<Decoder<BufReader<File>>>,
    stream: Arc<MixerDeviceSink>,
}

struct Bar {
    tempo: u32,
    unit: u32,
    nnotes: usize,
    accents: Vec<Click>,
}

struct Song {
    bars: Vec<Bar>,
    loops: Option<u32>,
}

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

impl Click {
    fn init(name: String, path: PathBuf, stream: Arc<MixerDeviceSink>) -> io::Result<Self> {
        let file = File::open(&path)?;
        let decoder = Decoder::try_from(file).expect("decoder");
        let buffer = decoder.buffered();

        Ok(Self {
            name,
            buffer,
            stream,
        })
    }

    fn play(&self) {
        self.stream.mixer().add(self.buffer.clone());
        print!("{}", self.name);
        let _ = io::stdout().flush();
    }
}

impl Bar {
    fn play(&self) {
        let tempo_millis: u64 = ((1000 * 60 / self.tempo) as f32 * (4.0 / self.unit as f32)) as u64;
        print!("|");
        let _ = io::stdout().flush();
        for note in 0..self.nnotes {
            if note >= self.accents.len() {
                self.accents
                    .last()
                    .expect("couldn't get last element")
                    .play();
            } else {
                self.accents[note].play();
            }
            std::thread::sleep(std::time::Duration::from_millis(tempo_millis));
        }
    }
}

impl Song {
    fn play(self) {
        match self.loops {
            Some(x) => {
                for _ in 0..x {
                    for bar in &self.bars {
                        bar.play()
                    }
                }
            }
            None => loop {
                for bar in &self.bars {
                    bar.play()
                }
            },
        }
        println!("|");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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
            Click::init(pid.into(), path, Arc::clone(&stream_handle)).expect("creating click");
        clicks.push(click);
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
    };

    song.play();
    Ok(())
}
