use std::time::Duration;

pub type MotionModel = Vec<Series>;

pub struct Sample {
    pub dt: Duration,
    pub acc_mps2: [f64; 3],
    pub gyr_dps: [f64; 3],
}
pub struct Series {
    pub reference: String,
    pub samples: Vec<Sample>,
}
impl Series {
    pub fn new(reference: String) -> Self {
        Self {
            reference,
            samples: Vec::new(),
        }
    }
}
