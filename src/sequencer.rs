use rodio::{Decoder, MixerDeviceSink, Source, source::Buffered};
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;

pub enum Event {
    Click(ClickKind),
    Bar(u32),
}

#[derive(Copy, Clone)]
pub enum ClickKind {
    Accent,
    Normal,
    Upbeat,
    Rest,
}

pub struct Click {
    pub name: String,
    pub buffer: Buffered<Decoder<BufReader<File>>>,
    pub stream: Arc<MixerDeviceSink>,
    pub tx: mpsc::Sender<Event>,
}

pub struct Clicks {
    accent: Click,
    normal: Click,
    upbeat: Click,
}

pub struct Bar {
    pub tempo: u32,
    pub unit: u32,
    pub nnotes: usize,
    pub accents: Vec<ClickKind>,
    pub clicks: Clicks,
    pub tx: mpsc::Sender<Event>,
}

pub struct Song {
    pub bars: Vec<Bar>,
    pub loops: Option<u32>,
    pub tx: mpsc::Sender<Event>,
}

impl fmt::Display for ClickKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClickKind::Accent => write!(f, "X"),
            ClickKind::Normal => write!(f, "x"),
            ClickKind::Upbeat => write!(f, "-"),
            ClickKind::Rest => write!(f, "."),
        }
    }
}

impl From<char> for ClickKind {
    fn from(value: char) -> Self {
        match value {
            'X' => ClickKind::Accent,
            'x' => ClickKind::Normal,
            '-' => ClickKind::Upbeat,
            _ => ClickKind::Rest,
        }
    }
}

impl Clicks {
    pub fn init(
        accent: PathBuf,
        normal: PathBuf,
        upbeat: PathBuf,
        tx: mpsc::Sender<Event>,
    ) -> io::Result<Self> {
        let stream_handle = Arc::new(rodio::DeviceSinkBuilder::open_default_sink().unwrap());

        Ok(Self {
            accent: Click::init(
                ClickKind::Accent.to_string(),
                accent,
                Arc::clone(&stream_handle),
                tx.clone(),
            )?,
            normal: Click::init(
                ClickKind::Normal.to_string(),
                normal,
                Arc::clone(&stream_handle),
                tx.clone(),
            )?,
            upbeat: Click::init(
                ClickKind::Upbeat.to_string(),
                upbeat,
                Arc::clone(&stream_handle),
                tx.clone(),
            )?,
        })
    }
    pub fn play(&self, kind: &ClickKind) {
        match kind {
            ClickKind::Accent => self.accent.play(),
            ClickKind::Normal => self.normal.play(),
            ClickKind::Upbeat => self.upbeat.play(),
            ClickKind::Rest => (),
        }
    }
}

impl Click {
    pub fn init(
        name: String,
        path: PathBuf,
        stream: Arc<MixerDeviceSink>,
        tx: mpsc::Sender<Event>,
    ) -> io::Result<Self> {
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
    }
}

impl Bar {
    pub fn play(&self) {
        let tempo_millis: u64 = ((1000 * 60 / self.tempo) as f32 * (4.0 / self.unit as f32)) as u64;
        let _ = self.tx.send(Event::Bar(42));
        for click in &self.accents {
            let _ = self.tx.send(Event::Click(*click));
            self.clicks.play(click);
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
