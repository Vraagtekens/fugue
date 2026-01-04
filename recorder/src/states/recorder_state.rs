use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

pub struct RecorderState {
    active_notes: HashSet<u8>,
    sustain_down: bool,
    last_activity: Instant,
}

impl RecorderState {
    pub fn new() -> Self {
        Self {
            active_notes: HashSet::new(),
            sustain_down: false,
            last_activity: Instant::now(),
        }
    }

    pub fn handle_midi(&mut self, msg: &[u8]) {
        let status = msg[0] & 0xF0;
        let key = msg.get(1).copied().unwrap_or(0);
        let val = msg.get(2).copied().unwrap_or(0);

        match status {
            // NOTE ON
            0x90 if val > 0 => {
                self.active_notes.insert(key);
                self.last_activity = Instant::now();
            }

            // NOTE OFF or NOTE ON vel=0
            0x80 | 0x90 => {
                self.active_notes.remove(&key);
                self.last_activity = Instant::now();
            }

            // SUSTAIN PEDAL (CC64)
            0xB0 if key == 64 => {
                self.sustain_down = val >= 64;
                self.last_activity = Instant::now();
            }

            _ => {}
        }
    }

    pub fn is_idle(&self, timeout: Duration) -> bool {
        self.active_notes.is_empty() && !self.sustain_down && self.last_activity.elapsed() > timeout
    }

    pub fn reset(&mut self) {
        self.active_notes.clear();
        self.sustain_down = false;
        self.last_activity = Instant::now();
    }
}
