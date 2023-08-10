pub struct Servo {
    range: (u64, u64),
    current: u64,
}

impl Servo {
    pub fn new(min: u64, max: u64) -> Servo {
        Servo {}
    }
}
