use std::{fmt::Display, time::Duration};

use amcx_core::{Model, Record, Sample};
use iced::Padding;
use iced::widget::{column, container, text};
use iced::{Border, Color, Element, Length::Fill, color};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use super::{Charts, Message};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorID {
    pub id: usize,
    pub name: String,
}
impl Display for SensorID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Charts {
    pub fn from_model(model: &Model) -> Charts {
        let mut sensors = Vec::with_capacity(model.len());
        let mut charts = Vec::with_capacity(model.len());

        for (id, (sensor, stream)) in model.into_iter().enumerate() {
            sensors.push(SensorID {
                id,
                name: sensor.clone(),
            });

            let (acc_stream, gyr_stream): (Vec<_>, Vec<_>) = stream
                .into_iter()
                .map(|record| {
                    let Record { timestamp, sample } = record;
                    let Sample { acc, gyr } = sample;
                    ((timestamp, acc), (timestamp, gyr))
                })
                .unzip();

            let acc_chart = ChartSensor::new(&acc_stream, Caption::Accelerometer);
            let gyr_chart = ChartSensor::new(&gyr_stream, Caption::Gyroscope);

            charts.push((acc_chart, gyr_chart));
        }

        let selected = sensors.first().cloned();
        Charts {
            sensors,
            selected,
            charts,
        }
    }
}

enum Caption {
    Accelerometer,
    Gyroscope,
}
impl Caption {
    fn str(&self) -> &'static str {
        match self {
            Caption::Accelerometer => "Accelerometer",
            Caption::Gyroscope => "Gyroscope",
        }
    }
}

pub struct ChartSensor {
    caption: Caption,
    time_range: (f32, f32),
    axis_range: (f32, f32),
    line_x: Vec<(f32, f32)>,
    line_y: Vec<(f32, f32)>,
    line_z: Vec<(f32, f32)>,
}

type DataPoints<'a> = (&'a Duration, &'a [f32; 3]);
impl ChartSensor {
    pub fn view(&self) -> Element<Message> {
        container(column![
            text(self.caption.str()).size(20).width(Fill).center(),
            ChartWidget::new(self).height(Fill).width(Fill)
        ])
        .padding(Padding::ZERO.top(10))
        .style(|_theme| container::Style {
            background: Some(color!(40, 40, 50).into()),
            border: Border {
                color: Color::WHITE,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
    fn new(samples: &[DataPoints], caption: Caption) -> Self {
        let mut line_x = Vec::with_capacity(samples.len());
        let mut line_y = Vec::with_capacity(samples.len());
        let mut line_z = Vec::with_capacity(samples.len());

        let mut min = f32::MAX;
        let mut max = f32::MIN;

        let mut ts = 0.0;
        for (timestamp, [x, y, z]) in samples {
            ts = timestamp.as_secs_f32();

            line_x.push((ts, *x));
            line_y.push((ts, *y));
            line_z.push((ts, *z));

            min = min.min(*x).min(*y).min(*z);
            max = max.max(*x).max(*y).max(*z);
        }

        Self {
            caption,
            line_x,
            line_y,
            line_z,
            time_range: (0.0, ts),
            axis_range: (min, max),
        }
    }
}
impl Chart<Message> for ChartSensor {
    type State = ();

    // TODO handle errors
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        use plotters::prelude::*;

        let x_spec = self.time_range.0..self.time_range.1;
        let y_spec = (self.axis_range.0 - 0.5)..(self.axis_range.1 + 0.5);
        let mut chart = builder
            .margin(20)
            .margin_top(10)
            .x_label_area_size(20)
            .y_label_area_size(40)
            .build_cartesian_2d(x_spec, y_spec)
            .expect("failed to build chart");
        chart
            .configure_mesh()
            .light_line_style(WHITE.mix(0.01))
            .bold_line_style(WHITE.mix(0.02))
            .axis_style(WHITE)
            .label_style(&WHITE)
            .max_light_lines(5)
            .draw()
            .expect("failed to draw mesh");

        let line_x = LineSeries::new(self.line_x.iter().cloned(), plotters::style::RED);
        let line_y = LineSeries::new(self.line_y.iter().cloned(), plotters::style::GREEN);
        let line_z = LineSeries::new(self.line_z.iter().cloned(), plotters::style::CYAN);

        chart.draw_series(line_x).expect("failed to draw series");
        chart.draw_series(line_y).expect("failed to draw series");
        chart.draw_series(line_z).expect("failed to draw series");
    }
}
