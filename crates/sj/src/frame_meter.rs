use std::time::{Duration, Instant};

const READING_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    pub emulated_fps: u32,
    pub top_screen_fps: u32,
}

// Counts emulated frames, and the frames the game actually drew on the top screen: a game frame
// is one tick of its main loop counter, weighted by the top screen's share of it.
pub struct FrameMeter {
    since: Instant,
    frames: u32,
    top_screen_frames: f32,
    last_loop_counter: Option<u32>,
}

impl FrameMeter {
    pub fn new(now: Instant) -> Self {
        Self {
            since: now,
            frames: 0,
            top_screen_frames: 0.0,
            last_loop_counter: None,
        }
    }

    pub fn record(&mut self, loop_counter: Option<u32>, top_screen_share: f32) {
        self.frames += 1;
        let game_frame = self.last_loop_counter.is_some() && loop_counter != self.last_loop_counter;
        if game_frame {
            self.top_screen_frames += top_screen_share;
        }
        self.last_loop_counter = loop_counter;
    }

    pub fn take_reading(&mut self, now: Instant) -> Option<Reading> {
        let elapsed = now.duration_since(self.since);
        if elapsed < READING_INTERVAL {
            return None;
        }
        let seconds = elapsed.as_secs_f32();
        let reading = Reading {
            emulated_fps: (self.frames as f32 / seconds).round() as u32,
            top_screen_fps: (self.top_screen_frames / seconds).round() as u32,
        };
        self.since = now;
        self.frames = 0;
        self.top_screen_frames = 0.0;
        Some(reading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn second_of(meter: &mut FrameMeter, start: Instant, share: f32, game_frame_every: u32) {
        for frame in 0..60 {
            meter.record(Some(frame / game_frame_every), share);
        }
        let reading = meter.take_reading(start + READING_INTERVAL);
        assert!(reading.is_some());
    }

    #[test]
    fn alternating_3d_reads_as_30_on_the_top_screen() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(start);
        for frame in 0..60 {
            meter.record(Some(frame), 0.5);
        }
        assert_eq!(meter.take_reading(start), None, "not a second yet");
        let reading = meter.take_reading(start + READING_INTERVAL).unwrap();
        assert_eq!(reading.emulated_fps, 60);
        assert_eq!(
            reading.top_screen_fps, 30,
            "59 game frames after the first sample"
        );
    }

    #[test]
    fn a_lagging_game_draws_fewer_frames_than_the_emulator_runs() {
        let start = Instant::now();
        let mut meter = FrameMeter::new(start);
        second_of(&mut meter, start, 1.0, 1);
        for frame in 0..60 {
            meter.record(Some(1000 + frame / 2), 1.0);
        }
        let reading = meter.take_reading(start + 2 * READING_INTERVAL).unwrap();
        assert_eq!((reading.emulated_fps, reading.top_screen_fps), (60, 30));
    }
}
