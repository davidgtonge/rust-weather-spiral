//! Daylight-hour detection and sunprint layout helpers.

use crate::state::{Metric, MetricSeries};

/// Shortwave radiation above this (W/m²) counts as daylight in the archive data.
pub const DAYLIGHT_THRESHOLD: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
pub struct DayWindow {
    pub first_hour: u32,
    pub last_hour: u32,
}

pub fn is_daylight_hour(series: &MetricSeries, index: u32) -> bool {
    series.value_at(Metric::Sunlight, index) > DAYLIGHT_THRESHOLD
}

pub fn count_daylight_hours(series: &MetricSeries) -> u32 {
    (0..series.hour_count())
        .filter(|&i| is_daylight_hour(series, i))
        .count() as u32
}

/// First and last lit hour-of-day (0–23) for a calendar day index, if any.
pub fn day_window(series: &MetricSeries, day_index: u32) -> Option<DayWindow> {
    let base = day_index * 24;
    if base >= series.hour_count() {
        return None;
    }
    let mut first = None;
    let mut last = None;
    for h in 0..24u32 {
        let idx = base + h;
        if idx >= series.hour_count() {
            break;
        }
        if is_daylight_hour(series, idx) {
            if first.is_none() {
                first = Some(h);
            }
            last = Some(h);
        }
    }
    match (first, last) {
        (Some(first_hour), Some(last_hour)) if first_hour <= last_hour => {
            Some(DayWindow {
                first_hour,
                last_hour,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::MetricSeries;

    fn series_with_sunlight(values: &[f32]) -> MetricSeries {
        let blob: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        MetricSeries::new(
            std::array::from_fn(|idx| {
                if idx == Metric::Sunlight.index() {
                    blob.clone()
                } else {
                    Vec::new()
                }
            }),
            values.len() as u32,
        )
    }

    #[test]
    fn day_window_finds_lit_span() {
        let mut hours = vec![0.0; 48];
        for h in 8..18 {
            hours[h] = 100.0;
        }
        let series = series_with_sunlight(&hours);
        let w = day_window(&series, 0).expect("day 0");
        assert_eq!(w.first_hour, 8);
        assert_eq!(w.last_hour, 17);
        assert_eq!(count_daylight_hours(&series), 10);
    }

    #[test]
    fn polar_night_day_is_empty() {
        let hours = vec![0.0; 24];
        let series = series_with_sunlight(&hours);
        assert!(day_window(&series, 0).is_none());
    }
}
