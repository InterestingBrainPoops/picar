pub struct MotionProfile {
    start_tick: u64,
    end_tick: u64,
    beginning: f64,
    end: f64,
}

impl MotionProfile {
    pub fn new(beginning: f64, end: f64, rate: f64, start: u64) -> MotionProfile {
        MotionProfile {
            beginning,
            end,
            start_tick: start,
            end_tick: start + ((end - beginning).abs() / rate) as u64,
        }
    }

    pub fn probe(&self, time: u64) -> f64 {
        assert!(time <= self.end_tick);
        self.beginning
            + ((time - self.start_tick) as f64 / (self.end_tick - self.start_tick) as f64)
                * (self.end - self.beginning)
    }

    pub fn done(&self, time: u64) -> bool {
        time > self.end_tick
    }
}
