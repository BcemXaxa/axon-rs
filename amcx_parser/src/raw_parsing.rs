use crate::parsing_error::*;
use amcx_core::raw::*;

enum ConfigKV {
    Bits(Bits),
    Clock(Clock),
    AccFS(AccelSR),
    GyroFS(GyroSR),
}
impl ConfigKV {
    fn parse(key: &str, value: &str) -> Result<ConfigKV, InnerParsingError> {
        use ConfigKey as K;
        match key {
            K::KEY_BITS => Bits::from_str(value)
                .map(Self::Bits)
                .ok_or_else(|| Self::unsupported_value(value, key, &Bits::ALL_VALUES)),

            K::KEY_CLOCK => Clock::from_str(value)
                .map(Self::Clock)
                .ok_or_else(|| Self::unsupported_value(value, key, &Clock::ALL_VALUES)),

            K::KEY_ACCEL_SR => AccelSR::from_str(value)
                .map(Self::AccFS)
                .ok_or_else(|| Self::unsupported_value(value, key, &AccelSR::ALL_VALUES)),

            K::KEY_GYRO_SR => GyroSR::from_str(value)
                .map(Self::GyroFS)
                .ok_or_else(|| Self::unsupported_value(value, key, &GyroSR::ALL_VALUES)),

            unknown => Err(InnerParsingError::ConfigUnknownKey(unknown.to_owned())),
        }
    }
    fn unsupported_value(value: &str, key: &str, valid: &[&'static str]) -> InnerParsingError {
        InnerParsingError::ConfigUnsupportedValue {
            value: value.to_owned(),
            key: key.to_owned(),
            valid_values: valid.to_owned(),
        }
    }
}

pub fn raw_parse(source: &str) -> Result<File, ParsingError> {
    let mut source = source
        .lines()
        .map(str::trim)
        .enumerate()
        .filter_map(|(i, s)| (!s.is_empty()).then_some((i + 1, s)));

    let config = block(parse_config, source.next(), ("?[", "]"))?;
    let sensors: Vec<Sensor> = block(parse_sensors, source.next(), ("&[", "]"))?;
    let mut clusters = Vec::new();

    while let Some((line, delta)) = source.next() {
        let delta = delta
            .parse()
            .map_err(|err| InnerParsingError::NumberParsing(err).at(line))?;
        let mut samples = Vec::with_capacity(sensors.len());

        for _ in 0..sensors.len() {
            let sample = block(parse_sample, source.next(), ("[", "]"))?;
            samples.push(sample);
        }

        clusters.push(Cluster { delta, samples });
    }

    Ok(File {
        config,
        sensors,
        clusters,
    })
}

fn block<F, O>(
    action: F,
    source: Option<(usize, &str)>,
    block: (&str, &str),
) -> Result<O, ParsingError>
where
    F: Fn(&str) -> Result<O, InnerParsingError>,
{
    let placeholder = format!("{}...{}", block.0, block.1);

    let (line, source) =
        source.ok_or(InnerParsingError::TokenExpected(placeholder.clone()).into())?;
    if !(source.starts_with(block.0) && source.ends_with(block.1)) {
        return Err(InnerParsingError::TokenUnexpected {
            expected: placeholder,
            found: source.into(),
        }
        .at(line));
    }

    let source = &source[(block.0.len())..(source.len() - block.1.len())];
    action(source).map_err(|err| err.at(line))
}

fn parse_config(source: &str) -> Result<Config, InnerParsingError> {
    let source = source
        .split_whitespace()
        .map(|s| s.split_once('=').ok_or(s));

    let mut bits = None;
    let mut clock = None;
    let mut acc_fs = None;
    let mut gyro_fs = None;

    for config in source {
        let config = config
            .map(|(key, value)| ConfigKV::parse(key, value))
            .map_err(|err| InnerParsingError::TokenUnexpected {
                expected: "KEY=VALUE".into(),
                found: err.to_owned(),
            })??;

        use ConfigKV as KV;
        use ConfigKey as K;
        match config {
            KV::Bits(value) => {
                if bits.replace(value).is_some() {
                    Err(InnerParsingError::ConfigDuplicate(K::KEY_BITS))?
                }
            }
            KV::Clock(value) => {
                if clock.replace(value).is_some() {
                    Err(InnerParsingError::ConfigDuplicate(K::KEY_CLOCK))?
                }
            }
            KV::GyroFS(value) => {
                if gyro_fs.replace(value).is_some() {
                    Err(InnerParsingError::ConfigDuplicate(K::KEY_GYRO_SR))?
                }
            }
            KV::AccFS(value) => {
                if acc_fs.replace(value).is_some() {
                    Err(InnerParsingError::ConfigDuplicate(K::KEY_ACCEL_SR))?
                }
            }
        }
    }

    use ConfigKey as Key;
    Ok(Config {
        bits: bits.ok_or(InnerParsingError::ConfigMissing(Key::KEY_BITS))?,
        clock: clock.ok_or(InnerParsingError::ConfigMissing(Key::KEY_CLOCK))?,
        gyro_sr: gyro_fs.ok_or(InnerParsingError::ConfigMissing(Key::KEY_GYRO_SR))?,
        accel_sr: acc_fs.ok_or(InnerParsingError::ConfigMissing(Key::KEY_ACCEL_SR))?,
    })
}

fn parse_sensors(source: &str) -> Result<Vec<Sensor>, InnerParsingError> {
    let mut source = source.split_whitespace();
    let mut sensors: Vec<Sensor> = Vec::new();
    while let Some(sensor) = source.next() {
        if sensors.iter().any(|s| s == sensor) {
            return Err(InnerParsingError::SensorNameDuplicate(sensor.into()));
        }
        sensors.push(sensor.into())
    }
    Ok(sensors)
}

fn parse_sample(source: &str) -> Result<Sample, InnerParsingError> {
    let mut source = source.split_whitespace();

    let mut sample: Sample = [0; 6];
    for raw in &mut sample {
        *raw = source
            .next()
            .ok_or(InnerParsingError::TokenExpected("number".into()))?
            .parse::<i32>()
            .map_err(|err| InnerParsingError::NumberParsing(err))?;
    }

    if let Some(what) = source.next() {
        return Err(InnerParsingError::TokenUnexpected {
            expected: "nothing".into(),
            found: what.into(),
        });
    }

    Ok(sample)
}
