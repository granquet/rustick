pub enum Event {
    Click(String),
    Bar(u32),
}

pub mod sequencer {
use crate::Event;
use rodio::{Decoder, MixerDeviceSink, Source, source::Buffered};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;

#[derive(Clone)]
    pub struct Click {
        pub name: String,
        pub buffer: Buffered<Decoder<BufReader<File>>>,
        pub stream: Arc<MixerDeviceSink>,
        pub tx: mpsc::Sender<Event>,
    }

    pub struct Bar {
        pub tempo: u32,
        pub unit: u32,
        pub nnotes: usize,
        pub accents: Vec<Click>,
        pub tx: mpsc::Sender<Event>,
    }

    pub struct Song {
        pub bars: Vec<Bar>,
        pub loops: Option<u32>,
        pub tx: mpsc::Sender<Event>,
    }

    impl Click {
        pub fn init(name: String, path: PathBuf, stream: Arc<MixerDeviceSink>, tx: mpsc::Sender<Event>) -> io::Result<Self> {
            let file = File::open(&path)?;
            let decoder = Decoder::try_from(file).expect("decoder");
            let buffer = decoder.buffered();

            Ok(Self {
                name,
                buffer,
                stream,
                tx,
            })
        }

        pub fn play(&self) {
            self.stream.mixer().add(self.buffer.clone());
            let _ = self.tx.send(Event::Click(self.name.clone()));
        }
    }

    impl Bar {
        pub fn play(&self) {
            let tempo_millis: u64 = ((1000 * 60 / self.tempo) as f32 * (4.0 / self.unit as f32)) as u64;
            let _ = self.tx.send(Event::Bar(42));
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
        pub fn play(self) {
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
            let _ = self.tx.send(Event::Bar(42));
        }
    }
}
