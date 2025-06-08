use std::time::Duration;

use amcx_core::raw::{Cluster, Config, File};
use amcx_core::{Model, Record, Sample, Stream};

use crate::{parsing_error::ParsingError, raw_parse};

pub fn parse(source: &str) -> Result<Model, ParsingError> {
    let File {
        config,
        sensors,
        clusters,
    } = raw_parse(source)?;

    let mut model: Model = sensors
        .into_iter()
        .map(|sensor| {
            let stream: Stream = Vec::with_capacity(clusters.len());
            (sensor, stream)
        })
        .collect();

    let mut timestamp = Duration::ZERO;
    for Cluster { delta, samples } in clusters {
        timestamp += config.clock.duration(delta);
        for (sample, (_, stream)) in samples.into_iter().zip(model.iter_mut()) {
            let record = Record {
                timestamp,
                sample: resolve_sample(sample, &config),
            };
            stream.push(record);
        }
    }

    Ok(model)
}

fn resolve_sample(sample: amcx_core::raw::Sample, config: &Config) -> Sample {
    let mut acc = [0.0; 3];
    let mut gyr = [0.0; 3];
    let chain = acc.iter_mut().chain(gyr.iter_mut()).zip(sample.into_iter());

    let invert_lsb = (1u32 << config.bits.as_u8()) as f32;
    for (prx, raw) in chain {
        *prx = raw as f32 / invert_lsb;
    }

    const G: f32 = 9.80665;
    for a in &mut acc {
        *a *= config.accel_sr.total_scale() * G;
    }
    for g in &mut gyr {
        *g *= config.gyro_sr.total_scale();
    }

    Sample { acc, gyr }
}
