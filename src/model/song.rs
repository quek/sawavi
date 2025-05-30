use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::track::Track;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub bpm: f64,
    pub sample_rate: f64,
    pub lpb: u16,
    pub play_p: bool,
    pub play_position: Range<i64>,
    pub tracks: Vec<Track>,
}

impl Song {
    pub fn new() -> Self {
        Self {
            bpm: 128.0,
            sample_rate: 48000.0,
            lpb: 4,
            play_p: false,
            play_position: (0..0),
            tracks: vec![],
        }
    }

    pub fn add_track(&mut self) {
        let mut track = Track::new();
        track.name = format!("T{:02X}", self.tracks.len() + 1);
        self.tracks.push(track);
    }
}
