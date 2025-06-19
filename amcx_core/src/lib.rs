pub use processed::*;

pub mod processed {
    use std::time::Duration;

    pub type Model = Vec<(Sensor, Stream)>;
    pub type Sensor = String;
    pub type Stream = Vec<Record>;

    #[derive(Debug, Clone)]
    pub struct Sample {
        // in g
        pub acc: [f32; 3],
        // in rad/s
        pub gyr: [f32; 3],
    }

    #[derive(Debug, Clone)]
    pub struct Record {
        pub timestamp: Duration,
        pub sample: Sample,
    }
}

pub mod raw {
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct File {
        pub config: Config,
        pub sensors: Vec<Sensor>,
        pub clusters: Vec<Cluster>,
    }

    pub type Sensor = String;
    pub type Sample = [i32; 6];

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Cluster {
        pub delta: u32,
        pub samples: Vec<Sample>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Config {
        pub bits: Bits,
        pub clock: Clock,
        pub accel_sr: AccelSR,
        pub gyro_sr: GyroSR,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Bits {
        _8,
        _16,
        _32,
    }
    impl Bits {
        pub const VALUE_8: &str = "8";
        pub const VALUE_16: &str = "16";
        pub const VALUE_32: &str = "32";

        pub const ALL_VALUES: [&str; 3] = [Self::VALUE_8, Self::VALUE_16, Self::VALUE_32];

        pub fn from_str(val: &str) -> Option<Self> {
            Some(match val {
                Self::VALUE_8 => Self::_8,
                Self::VALUE_16 => Self::_16,
                Self::VALUE_32 => Self::_32,
                _ => return None,
            })
        }

        pub const fn as_u8(&self) -> u8 {
            match self {
                Self::_8 => 8,
                Self::_16 => 16,
                Self::_32 => 32,
            }
        }
    }

    // in g
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum AccelSR {
        _2,
        _4,
        _8,
        _16,
    }
    impl AccelSR {
        pub const VALUE_2: &str = "2";
        pub const VALUE_4: &str = "4";
        pub const VALUE_8: &str = "8";
        pub const VALUE_16: &str = "16";

        pub const ALL_VALUES: [&str; 4] =
            [Self::VALUE_2, Self::VALUE_4, Self::VALUE_8, Self::VALUE_16];

        pub fn from_str(val: &str) -> Option<Self> {
            Some(match val {
                Self::VALUE_2 => Self::_2,
                Self::VALUE_4 => Self::_4,
                Self::VALUE_8 => Self::_8,
                Self::VALUE_16 => Self::_16,
                _ => return None,
            })
        }

        pub const fn total_scale_g(&self) -> f32 {
            match self {
                Self::_2 => 4.0,
                Self::_4 => 8.0,
                Self::_8 => 16.0,
                Self::_16 => 32.0,
            }
        }

        pub const fn total_scale_m(&self) -> f32 {
            const G: f32 = 9.80665;
            self.total_scale_g() * G
        }
    }

    // in deg/s
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum GyroSR {
        _250,
        _500,
        _1000,
        _2000,
    }
    impl GyroSR {
        pub const VALUE_250: &str = "250";
        pub const VALUE_500: &str = "500";
        pub const VALUE_1000: &str = "1000";
        pub const VALUE_2000: &str = "2000";

        pub const ALL_VALUES: [&str; 4] = [
            Self::VALUE_250,
            Self::VALUE_500,
            Self::VALUE_1000,
            Self::VALUE_2000,
        ];

        pub fn from_str(val: &str) -> Option<Self> {
            Some(match val {
                Self::VALUE_250 => Self::_250,
                Self::VALUE_500 => Self::_500,
                Self::VALUE_1000 => Self::_1000,
                Self::VALUE_2000 => Self::_2000,
                _ => return None,
            })
        }

        pub const fn total_scale_deg(&self) -> f32 {
            match self {
                Self::_250 => 500.0,
                Self::_500 => 1000.0,
                Self::_1000 => 2000.0,
                Self::_2000 => 4000.0,
            }
        }

        pub const fn total_scale_rad(&self) -> f32 {
            self.total_scale_deg().to_radians()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Clock {
        Milli,
        Micro,
    }
    impl Clock {
        pub const VALUE_MILLI: &str = "milli";
        pub const VALUE_MICRO: &str = "micro";

        pub const ALL_VALUES: [&str; 2] = [Self::VALUE_MILLI, Self::VALUE_MICRO];

        pub fn from_str(val: &str) -> Option<Self> {
            Some(match val.to_ascii_lowercase().as_str() {
                Self::VALUE_MILLI => Self::Milli,
                Self::VALUE_MICRO => Self::Micro,
                _ => return None,
            })
        }

        pub const fn duration(&self, val: u32) -> Duration {
            match self {
                Self::Milli => Duration::from_millis(val as u64),
                Self::Micro => Duration::from_micros(val as u64),
            }
        }
    }

    pub struct ConfigKey;
    impl ConfigKey {
        pub const KEY_BITS: &str = "BITS";
        pub const KEY_CLOCK: &str = "CLOCK";
        pub const KEY_ACCEL_SR: &str = "ACCEL_SR";
        pub const KEY_GYRO_SR: &str = "GYRO_SR";

        pub const ALL_KEYS: [&str; 4] = [
            Self::KEY_BITS,
            Self::KEY_CLOCK,
            Self::KEY_ACCEL_SR,
            Self::KEY_GYRO_SR,
        ];
    }
}
