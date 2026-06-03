use serde::{Deserialize, Serialize};

use crate::view_mode::ViewMode;

pub const FRAME_SIZE: u32 = 1024;
pub const HOUR_STEP: u32 = 3600;

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(rename_all = "camelCase"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Metric {
    Cloud,
    Sunlight,
    Rain,
    Wind,
    Temperature,
}

impl Metric {
    pub const ALL: [Metric; 5] = [
        Metric::Cloud,
        Metric::Sunlight,
        Metric::Rain,
        Metric::Wind,
        Metric::Temperature,
    ];

    pub const fn index(self) -> usize {
        match self {
            Metric::Cloud => 0,
            Metric::Sunlight => 1,
            Metric::Rain => 2,
            Metric::Wind => 3,
            Metric::Temperature => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Metric::Cloud => "Cloud",
            Metric::Sunlight => "Sunlight",
            Metric::Rain => "Rain",
            Metric::Wind => "Wind",
            Metric::Temperature => "Temperature",
        }
    }

    pub fn series_key(self) -> &'static str {
        match self {
            Metric::Cloud => "cloud",
            Metric::Sunlight => "sunlight",
            Metric::Rain => "rain",
            Metric::Wind => "wind",
            Metric::Temperature => "temperature",
        }
    }
}

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(rename_all = "camelCase"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Zoom {
    Year,
    Month,
    Week,
    Day,
}

impl Zoom {
    pub const ALL: [Zoom; 4] = [Zoom::Year, Zoom::Month, Zoom::Week, Zoom::Day];

    pub fn label(self) -> &'static str {
        match self {
            Zoom::Year => "Year",
            Zoom::Month => "Month",
            Zoom::Week => "Week",
            Zoom::Day => "Day",
        }
    }
}

/// Dense little-endian f32 bytes from CBOR — `hour_count * 4` per metric.
#[derive(Debug, Clone)]
pub struct MetricSeries {
    blobs: [Vec<u8>; 5],
    hour_count: u32,
}

impl MetricSeries {
    pub fn new(blobs: [Vec<u8>; 5], hour_count: u32) -> Self {
        Self { blobs, hour_count }
    }

    pub fn hour_count(&self) -> u32 {
        self.hour_count
    }

    pub fn bytes(&self, metric: Metric) -> &[u8] {
        &self.blobs[metric.index()]
    }

    pub fn value_at(&self, metric: Metric, index: u32) -> f32 {
        read_f32_le(self.bytes(metric), index)
    }
}

#[inline]
pub fn read_f32_le(blob: &[u8], index: u32) -> f32 {
    let off = (index as usize) * 4;
    f32::from_le_bytes([
        blob[off],
        blob[off + 1],
        blob[off + 2],
        blob[off + 3],
    ])
}

#[derive(Debug, Clone)]
pub struct CityWeather {
    pub id: String,
    pub label: String,
    pub start_unix: u32,
    pub hour_step: u32,
    pub metrics: MetricSeries,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub cities: Vec<CityWeather>,
    pub selected_city_id: String,
    pub selected_metric: Metric,
    pub selected_zoom: Zoom,
    pub selected_view_mode: ViewMode,
}

impl AppState {
    pub fn initial(cities: Vec<CityWeather>) -> Self {
        let selected_city_id = cities
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| "bristol".to_string());
        Self {
            cities,
            selected_city_id,
            selected_metric: Metric::Cloud,
            selected_zoom: Zoom::Year,
            selected_view_mode: ViewMode::Metric,
        }
    }

    pub fn selected_city(&self) -> Option<&CityWeather> {
        self.cities.iter().find(|c| c.id == self.selected_city_id)
    }
}
