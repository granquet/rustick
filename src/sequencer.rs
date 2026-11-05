use rodio::{ChannelCount, Decoder, MixerDeviceSink, Sample, SampleRate, Source, source::Buffered};
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

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
}

pub struct Song {
    pub bars: Vec<Bar>,
    pub loops: Option<u32>,
    pub clicks: Clicks,
}

pub struct Sequencer {
    sr: u32,
    pos: u64,
    next_click: f64,
    song: Song,
    bar: usize,
    accent: usize,
    sink: Arc<MixerDeviceSink>,
    tx: mpsc::Sender<Event>,
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
    pub fn init(accent: PathBuf, normal: PathBuf, upbeat: PathBuf) -> io::Result<Self> {
        Ok(Self {
            accent: Click::init(ClickKind::Accent.to_string(), accent)?,
            normal: Click::init(ClickKind::Normal.to_string(), normal)?,
            upbeat: Click::init(ClickKind::Upbeat.to_string(), upbeat)?,
        })
    }
    pub fn get(&self, kind: &ClickKind) -> Option<Buffered<Decoder<BufReader<File>>>> {
        match kind {
            ClickKind::Accent => Some(self.accent.get()),
            ClickKind::Normal => Some(self.normal.get()),
            ClickKind::Upbeat => Some(self.upbeat.get()),
            ClickKind::Rest => None,
        }
    }
}

impl Click {
    pub fn init(name: String, path: PathBuf) -> io::Result<Self> {
        let file = File::open(&path)?;
        let decoder = Decoder::try_from(file).expect("decoder");
        let buffer = decoder.buffered();

        Ok(Self { name, buffer })
    }

    pub fn get(&self) -> Buffered<Decoder<BufReader<File>>> {
        self.buffer.clone()
    }
}

impl Song {
    pub fn get(&self, bar: usize, accent: usize) -> &ClickKind {
        self.bars.get(bar).unwrap().accents.get(accent).unwrap()
    }
}

impl Iterator for Sequencer {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        if self.pos as f64 >= self.next_click {
            let click_kind = self.song.get(self.bar, self.accent);
            if self.accent == 0 {
                let _ = self.tx.send(Event::Bar(self.bar as u32));
            }

            let _ = self.tx.send(Event::Click(*click_kind));

            if let Some(click) = self.song.clicks.get(click_kind) {
                self.sink.mixer().add(click);
            }

            self.accent = (self.accent + 1) % self.song.bars.get(self.bar).unwrap().accents.len();
            let bpm = self.song.bars.get(self.bar).unwrap().tempo as f64;
            self.next_click += self.sr as f64 * 60.0 / bpm;
        }
        self.pos += 1;
        Some(0.0) // silence: the clicks come from the queued sources
    }
}

impl Source for Sequencer {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        ChannelCount::new(1).unwrap()
    }
    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(self.sr).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Sequencer {
    pub fn new(song: Song, tx: mpsc::Sender<Event>) -> Self {
        let sink = Arc::new(rodio::DeviceSinkBuilder::open_default_sink().unwrap());
        let sr = sink.config().sample_rate().get(); // match the device: no resampling
        Self {
            sr,
            pos: 0,
            next_click: 0.0,
            song,
            bar: 0,
            accent: 0,
            sink: Arc::clone(&sink),
            tx,
        }
    }

    pub fn run(self) {
        self.sink.clone().mixer().add(self);
        std::thread::park();
    }
}
