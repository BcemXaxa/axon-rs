use std::{fmt::Display, num::ParseIntError, time::Duration};

use thiserror::Error;

struct Sample {
    dt: Duration,
    acc_mps2: [f64; 3],
    gyr_dps: [f64; 3],
}

struct Series {
    reference: String,
    samples: Vec<Sample>,
}
impl Series {
    fn new(reference: String) -> Self {
        Self {
            reference,
            samples: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct Config {
    bits: u32,
    clock: Clock,
    gyro_fs: f64,
    acc_fs: f64,
}
impl Config {
    const BITS_DEFAULT: u32 = 16;
}
enum ConfigKeyValue {
    Bits(u32),
    Clock(Clock),
    GyroFS(f64),
    AccFS(f64),
}
impl ConfigKeyValue {
    const BITS: &str = "BITS";
    const CLOCK: &str = "CLOCK";
    const GYRO_FS: &str = "GYRO_FS";
    const ACC_FS: &str = "ACC_FS";

    fn parse<'a>(key: &'a str, value: &'a str) -> Result<ConfigKeyValue, InnerParsingError<'a>> {
        use ConfigKeyValue as Key;
        use ConfigKeyValue as Value;
        match key {
            Key::ACC_FS => Value::acc_fs(value).map(Self::AccFS).ok_or(
                InnerParsingError::ConfigUnsupportedValue {
                    value,
                    key,
                    valid_values: "2, 4, 8, 16",
                },
            ),
            Key::GYRO_FS => Value::gyro_fs(value).map(Self::GyroFS).ok_or(
                InnerParsingError::ConfigUnsupportedValue {
                    value,
                    key,
                    valid_values: "250, 500, 1000, 2000",
                },
            ),
            Key::BITS => Value::bits(value).map(Self::Bits).ok_or(
                InnerParsingError::ConfigUnsupportedValue {
                    value,
                    key,
                    valid_values: "8, 16, 32, 64",
                },
            ),
            Key::CLOCK => Value::clock(value).map(Self::Clock).ok_or(
                InnerParsingError::ConfigUnsupportedValue {
                    value,
                    key,
                    valid_values: "milli, micro",
                },
            ),
            unknown => Err(InnerParsingError::ConfigUnknownKey(unknown)),
        }
    }
    fn bits(val: &str) -> Option<u32> {
        match val {
            "8" => Some(8),
            "16" => Some(16),
            "32" => Some(32),
            "64" => Some(64),
            _ => None,
        }
    }
    fn clock(val: &str) -> Option<Clock> {
        match val {
            "micro" => Some(Clock::Microseconds),
            "milli" => Some(Clock::Milliseconds),
            _ => None,
        }
    }
    fn acc_fs(val: &str) -> Option<f64> {
        match val {
            "2" => Some(2.0),
            "4" => Some(4.0),
            "8" => Some(8.0),
            "16" => Some(16.0),
            _ => None,
        }
    }
    fn gyro_fs(val: &str) -> Option<f64> {
        match val {
            "250" => Some(250.0),
            "500" => Some(500.0),
            "1000" => Some(1000.0),
            "2000" => Some(2000.0),
            _ => None,
        }
    }
}

impl<'a> TryFrom<&'a str> for Config {
    type Error = InnerParsingError<'a>;

    fn try_from(source: &'a str) -> Result<Self, Self::Error> {
        let source = source
            .split_whitespace()
            .map(|s| s.split_once('=').ok_or(s));

        let mut bits = None;
        let mut clock = None;
        let mut gyro_fs = None;
        let mut acc_fs = None;

        for config in source {
            let config = config
                .map(|(key, value)| ConfigKeyValue::parse(key, value))
                .map_err(|err| InnerParsingError::TokenUnexpected {
                    expected: "KEY=VALUE",
                    found: err,
                })??;

            use ConfigKeyValue as KV;
            use ConfigKeyValue as Key;
            match config {
                KV::Bits(value) => {
                    if bits.is_some() {
                        return Err(InnerParsingError::ConfigDuplicate(Key::BITS));
                    }
                    bits.replace(value);
                }
                KV::Clock(value) => {
                    if clock.is_some() {
                        return Err(InnerParsingError::ConfigDuplicate(Key::CLOCK));
                    }
                    clock.replace(value);
                }
                KV::GyroFS(value) => {
                    if gyro_fs.is_some() {
                        return Err(InnerParsingError::ConfigDuplicate(Key::GYRO_FS));
                    }
                    gyro_fs.replace(value);
                }
                KV::AccFS(value) => {
                    if acc_fs.is_some() {
                        return Err(InnerParsingError::ConfigDuplicate(Key::ACC_FS));
                    }
                    acc_fs.replace(value);
                }
            }
        }

        use ConfigKeyValue as Key;
        Ok(Config {
            bits: bits.unwrap_or(Config::BITS_DEFAULT),
            clock: clock.ok_or(InnerParsingError::ConfigMissing(Key::CLOCK))?,
            gyro_fs: gyro_fs.ok_or(InnerParsingError::ConfigMissing(Key::GYRO_FS))?,
            acc_fs: acc_fs.ok_or(InnerParsingError::ConfigMissing(Key::ACC_FS))?,
        })
    }
}
#[derive(PartialEq, Eq, Debug)]
enum Clock {
    Milliseconds,
    Microseconds,
}
impl Clock {
    fn to_duration(&self, value: u64) -> Duration {
        match self {
            Clock::Milliseconds => Duration::from_millis(value),
            Clock::Microseconds => Duration::from_micros(value),
        }
    }
}

#[derive(Error, Debug)]
struct ParsingError<'a> {
    line: Option<usize>,
    inner: InnerParsingError<'a>,
}
impl<'a> Display for ParsingError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = &self.line {
            write!(f, "Line {line}: ")?
        }
        self.inner.fmt(f)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
enum InnerParsingError<'a> {
    #[error("expected {0}, but found nothing")]
    TokenExpected(&'static str),
    #[error("expected {expected}, but found {found}")]
    TokenUnexpected {
        expected: &'static str,
        found: &'a str,
    },
    #[error("duplicate references are not allowed: {0}")]
    ReferenceDuplicate(String),
    #[error("duplicate configs are not allowed: {0}")]
    ConfigDuplicate(&'static str),
    #[error("missing required config: {0}")]
    ConfigMissing(&'static str),
    #[error("unknown config key: {0}")]
    ConfigUnknownKey(&'a str),
    #[error("unsupported value {value} for key {key}, valid values are: {valid_values}")]
    ConfigUnsupportedValue {
        value: &'a str,
        key: &'a str,
        valid_values: &'static str,
    },
    #[error(transparent)]
    NumberParsing(#[from] ParseIntError),
}
impl<'a> InnerParsingError<'a> {
    fn at(self, line: usize) -> ParsingError<'a> {
        ParsingError {
            line: Some(line),
            inner: self,
        }
    }
}
impl<'a> Into<ParsingError<'a>> for InnerParsingError<'a> {
    fn into(self) -> ParsingError<'a> {
        ParsingError {
            line: None,
            inner: self,
        }
    }
}

fn parse(source: &str) -> Result<Vec<Series>, ParsingError> {
    let mut source = source
        .lines()
        .map(str::trim)
        .enumerate()
        .filter_map(|(i, s)| (!s.is_empty()).then_some((i + 1, s)));

    let config = parse_config(source.next())?;
    let references = parse_references(source.next())?;

    let mut reference_series: Vec<_> = references.into_iter().map(Series::new).collect();

    while let Some((line, duration_source)) = source.next() {
        let duration =
            parse_duration(duration_source, &config.clock).map_err(|err| err.at(line))?;

        for reference in reference_series.iter_mut() {
            reference
                .samples
                .push(parse_sample(source.next(), duration, &config)?);
        }
    }

    Ok(reference_series)
}

fn parse_config(source: Option<(usize, &str)>) -> Result<Config, ParsingError> {
    let (line, source) = source.ok_or(InnerParsingError::TokenExpected("?[...]").into())?;
    if !(source.starts_with("?[") && source.ends_with("]")) {
        return Err(InnerParsingError::TokenUnexpected {
            expected: "?[...]",
            found: source.into(),
        }
        .at(line));
    }
    let source = &source[2..source.len() - 1];

    Config::try_from(source).map_err(|err| err.at(line))
}

fn parse_references(source: Option<(usize, &str)>) -> Result<Vec<String>, ParsingError> {
    let (line, source) = source.ok_or(InnerParsingError::TokenExpected("&[...]").into())?;
    if !(source.starts_with("&[") && source.ends_with("]")) {
        return Err(InnerParsingError::TokenUnexpected {
            expected: "&[...]",
            found: source.into(),
        }
        .at(line));
    }

    let mut source = source[2..source.len() - 1].split_whitespace();
    let mut references: Vec<String> = Vec::new();
    while let Some(reference) = source.next() {
        if references.iter().any(|s| s == reference) {
            return Err(InnerParsingError::ReferenceDuplicate(reference.into()).at(line));
        }
        references.push(reference.to_string())
    }
    Ok(references)
}

fn parse_duration<'a>(source: &'a str, clock: &Clock) -> Result<Duration, InnerParsingError<'a>> {
    let raw = source
        .parse::<u64>()
        .map_err(InnerParsingError::NumberParsing)?;
    Ok(clock.to_duration(raw))
}

fn parse_sample<'a>(
    source: Option<(usize, &'a str)>,
    duration: Duration,
    config: &Config,
) -> Result<Sample, ParsingError<'a>> {
    let (line, source) = source.ok_or(InnerParsingError::TokenExpected("[...]").into())?;
    if !(source.starts_with("[") && source.ends_with("]")) {
        return Err(InnerParsingError::TokenUnexpected {
            expected: "[...]",
            found: source.into(),
        }
        .at(line));
    }

    let mut source = source[1..source.len() - 1].split_whitespace();
    let fs = (1 >> config.bits) as f64;
    const G: f64 = 9.80665;

    let mut acc = [0.0; 3];
    let mut gyr = [0.0; 3];
    let chain = acc.iter_mut().chain(gyr.iter_mut());
    for normalized in chain {
        let raw = source
            .next()
            .ok_or(InnerParsingError::TokenExpected("number").at(line))?
            .parse::<u64>()
            .map_err(|err| InnerParsingError::NumberParsing(err).at(line))?
            as f64;
        *normalized = (raw / fs) - 1.0;
    }
    // to m/(s^2)
    acc.iter_mut().for_each(|a| *a *= config.acc_fs * G);
    // to dps
    gyr.iter_mut().for_each(|g| *g *= config.gyro_fs);

    if let Some(what) = source.next() {
        return Err(InnerParsingError::TokenUnexpected {
            expected: "`]`".into(),
            found: what.into(),
        }
        .at(line));
    }

    Ok(Sample {
        dt: duration,
        acc_mps2: acc,
        gyr_dps: gyr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn config_valid() {
        let valid_config = "?[ BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
        let config = parse_config(Some((0, valid_config))).unwrap();
        assert_eq!(config.bits, 16);
        assert_eq!(config.clock, Clock::Microseconds);
        assert_relative_eq!(config.acc_fs, 2.0);
        assert_relative_eq!(config.gyro_fs, 250.0);

        let valid_config_spacing = "?[BITS=16   ACC_FS=2    GYRO_FS=250 CLOCK=micro]";
        let config = parse_config(Some((0, valid_config_spacing))).unwrap();
        assert_eq!(config.bits, 16);
        assert_eq!(config.clock, Clock::Microseconds);
        assert_relative_eq!(config.acc_fs, 2.0);
        assert_relative_eq!(config.gyro_fs, 250.0);

        let valid_config_default_bits = "?[ ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
        let config = parse_config(Some((0, valid_config_default_bits))).unwrap();
        assert_eq!(config.bits, Config::BITS_DEFAULT);
        assert_eq!(config.clock, Clock::Microseconds);
        assert_relative_eq!(config.acc_fs, 2.0);
        assert_relative_eq!(config.gyro_fs, 250.0);
    }
    #[test]
    fn config_invalid() {
        let invalid_block = "[ BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
        let invalid_duplicate = "?[ BITS=16 ACC_FS=2 GYRO_FS=250 BITS=16 CLOCK=micro ]";
        let invalid_key_value = "?[ UNKNOWN:10 BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
        let invalid_key = "?[ UNKNOWN=10 BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
        let invalid_bits = "?[ BITS=15 ACC_FS=2 GYRO_FS=250 CLOCK=micro ]";
        let invalid_acc = "?[ BITS=16 ACC_FS=3 GYRO_FS=250 CLOCK=micro ]";
        let invalid_gyro = "?[ BITS=16 ACC_FS=2 GYRO_FS=20 CLOCK=micro ]";
        let invalid_clock = "?[ BITS=16 ACC_FS=2 GYRO_FS=250 CLOCK=sec ]";
        let invalid_missing = "?[ BITS=16 GYRO_FS=250 CLOCK=micro ]";

        let invalid = [
            invalid_block,
            invalid_duplicate,
            invalid_key_value,
            invalid_key,
            invalid_bits,
            invalid_acc,
            invalid_gyro,
            invalid_clock,
            invalid_missing,
        ];

        for inv in invalid {
            assert!(parse_config(Some((1, inv))).is_err())
        }
    }

    #[test]
    fn reference_valid() {
        todo!()
    }

    #[test]
    fn reference_invalid() {
        todo!()
    }

    #[test]
    fn sample_valid() {
        todo!()
    }

    #[test]
    fn sample_invalid() {
        todo!()
    }

    #[test]
    fn duration_valid() {
        todo!()
    }

    #[test]
    fn duration_invalid() {
        todo!()
    }

    #[test]
    fn parsing_valid() {
        todo!()
    }

    #[test]
    fn parsing_invalid() {
        todo!()
    }
}
