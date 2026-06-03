use serde::Deserialize;

use crate::state::{CityWeather, Metric, MetricSeries, HOUR_STEP};

/// Embedded CBOR bundle — built by `npm run build:weather-cbor`.
const WEATHER_BUNDLE: &[u8] = include_bytes!("../../../data/weather.bundle.cbor");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeatherBundleFile {
    version: u8,
    hour_step: u32,
    cities: Vec<CityBundleFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CityBundleFile {
    id: String,
    label: String,
    start_unix: u32,
    hour_count: u32,
    #[serde(with = "serde_bytes")]
    cloud: Vec<u8>,
    #[serde(with = "serde_bytes")]
    sunlight: Vec<u8>,
    #[serde(with = "serde_bytes")]
    rain: Vec<u8>,
    #[serde(with = "serde_bytes")]
    wind: Vec<u8>,
    #[serde(with = "serde_bytes")]
    temperature: Vec<u8>,
}

pub fn load_embedded_cities() -> Result<Vec<CityWeather>, String> {
    let bundle: WeatherBundleFile =
        ciborium::from_reader(WEATHER_BUNDLE).map_err(|e| format!("cbor bundle: {e}"))?;

    if bundle.version != 1 {
        return Err(format!("unsupported bundle version {}", bundle.version));
    }
    if bundle.hour_step != HOUR_STEP {
        return Err(format!(
            "hourStep mismatch: {} (expected {HOUR_STEP})",
            bundle.hour_step
        ));
    }

    bundle
        .cities
        .into_iter()
        .map(|city| parse_city_bundle(city, bundle.hour_step))
        .collect()
}

fn parse_city_bundle(city: CityBundleFile, hour_step: u32) -> Result<CityWeather, String> {
    let blobs = [
        ("cloud", city.cloud),
        ("sunlight", city.sunlight),
        ("rain", city.rain),
        ("wind", city.wind),
        ("temperature", city.temperature),
    ];

    let expected = (city.hour_count as usize)
        .checked_mul(4)
        .ok_or_else(|| format!("{}: hour_count overflow", city.id))?;

    let mut arrays: [Vec<u8>; 5] = std::array::from_fn(|_| Vec::new());
    for (metric, (name, bytes)) in Metric::ALL.iter().zip(blobs) {
        if bytes.len() != expected {
            return Err(format!(
                "{}.{name}: expected {expected} bytes, got {}",
                city.id,
                bytes.len()
            ));
        }
        arrays[metric.index()] = bytes;
    }

    Ok(CityWeather {
        id: city.id,
        label: city.label,
        start_unix: city.start_unix,
        hour_step,
        metrics: MetricSeries::new(arrays, city.hour_count),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::read_f32_le;
    use std::time::Instant;

    #[test]
    fn embedded_cbor_loads_four_cities() {
        let cities = load_embedded_cities().expect("embedded cbor");
        assert_eq!(cities.len(), 4);
        let bristol = cities.iter().find(|c| c.id == "bristol").expect("bristol");
        assert_eq!(bristol.metrics.hour_count(), 8784);
        let v = read_f32_le(bristol.metrics.bytes(Metric::Cloud), 0);
        assert!(v.is_finite());
    }

    #[test]
    fn cbor_load_under_100ms() {
        let start = Instant::now();
        let cities = load_embedded_cities().expect("load");
        assert_eq!(cities.len(), 4);
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "load took {:?} (budget 100ms)",
            elapsed
        );
    }
}
