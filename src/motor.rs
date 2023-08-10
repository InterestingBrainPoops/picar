use std::time::Duration;

use rppal::pwm::{Channel, Polarity, Pwm};

pub struct Motor {
    pwm: Pwm,
    current_speed: u64,
    bounds: (u64, u64),
}

impl Motor {
    pub fn new(channel: Channel, bounds: (u64, u64)) -> Motor {
        Motor {
            pwm: Pwm::with_period(
                channel,
                Duration::from_millis(20),
                Duration::from_micros(bounds.0),
                Polarity::Normal,
                true,
            )
            .unwrap(),
            current_speed: bounds.0,
            bounds,
        }
    }

    pub fn warmup(&mut self, width: u64) {
        self.pwm
            .set_pulse_width(Duration::from_micros(width))
            .unwrap();
    }
    pub fn disable(&mut self) {
        self.pwm.disable().unwrap();
    }
    pub fn set_speed(&mut self, percentage: f64) {
        self.current_speed =
            self.bounds.0 + ((self.bounds.1 - self.bounds.0) as f64 * percentage) as u64;
        self.pwm
            .set_pulse_width(Duration::from_micros(self.current_speed))
            .unwrap();
    }
    pub fn set_duty(&mut self, duty: f64) {
        self.pwm.set_duty_cycle(duty).unwrap();
    }
    pub fn speed(&self) -> f64 {
        let range = (self.bounds.1 - self.bounds.0) as f64;
        let x = self.current_speed - self.bounds.0;
        x as f64 / range
    }

    pub fn set_pulse(&self, dur: Duration) {
        self.pwm.set_pulse_width(dur);
    }
}
