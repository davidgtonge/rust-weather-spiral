//! Per-view geometry encoders — colours, overlays, and draw commands.

use crate::colour::{colour_for, lerp_rgb, metric_domain, Rgba};
use crate::daylight::{day_window, is_daylight_hour};
use crate::draw::{
    push_condition_view, push_glyphs, push_sky_only, push_tapestry_layers, DrawWriter, SegmentDraw,
    SegmentPose, FLAG_BRIGHT, FLAG_STORM,
};
use crate::num::{f32_from_u32, f32_from_usize, u8_from_alpha, u8_from_f32_rounded};
use crate::spiral::{
    daylight_layout, daylight_point, fingerprint_layout, fingerprint_point, layout_ctx,
    layout_point, layout_segment_size, mandala_layout, mandala_point, ribbon_track_offset,
    tapestry_layout_ctx, tapestry_segment_size,
};
use crate::state::{read_f32_le, Metric, MetricSeries, Zoom};
use crate::view_mode::ViewMode;

const CLOUD_GREY: (u8, u8, u8) = (0xb0, 0xb8, 0xc4);
const SUN_YELLOW: (u8, u8, u8) = (0xf5, 0xc8, 0x42);
const STORM_INDIGO: (u8, u8, u8) = (0x1a, 0x1f, 0x4a);

pub struct WeatherSample {
    pub cloud: f32,
    pub sunlight: f32,
    pub rain: f32,
    pub wind: f32,
    pub temperature: f32,
}

impl WeatherSample {
    pub fn from_series(series: &MetricSeries, index: u32) -> Self {
        Self {
            cloud: series.value_at(Metric::Cloud, index),
            sunlight: series.value_at(Metric::Sunlight, index),
            rain: series.value_at(Metric::Rain, index),
            wind: series.value_at(Metric::Wind, index),
            temperature: series.value_at(Metric::Temperature, index),
        }
    }
}

fn norm(value: f32, metric: Metric) -> f32 {
    let (min, max) = metric_domain(metric);
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

fn scale_u8(value: f32, metric: Metric) -> u8 {
    u8_from_f32_rounded(norm(value, metric) * 255.0)
}

fn sky_rgba(sunlight: f32, cloud: f32) -> Rgba {
    let sun_t = norm(sunlight, Metric::Sunlight);
    let cloud_t = norm(cloud, Metric::Cloud);
    let (r, g, b) = lerp_rgb(sun_t, CLOUD_GREY, SUN_YELLOW);
    let mix = cloud_t * 0.65;
    Rgba {
        r: lerp_u8(mix, r, CLOUD_GREY.0),
        g: lerp_u8(mix, g, CLOUD_GREY.1),
        b: lerp_u8(mix, b, CLOUD_GREY.2),
        a: u8_from_alpha(1.0 - cloud_t * 0.45),
    }
}

fn lerp_u8(t: f32, a: u8, b: u8) -> u8 {
    let af = f32::from(a);
    let bf = f32::from(b);
    u8_from_f32_rounded(af + (bf - af) * t)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Condition {
    Sunny,
    Cloudy,
    Rainy,
    Windy,
    Stormy,
    Mixed,
}

impl Condition {
    fn classify(w: &WeatherSample) -> Self {
        let rain = norm(w.rain, Metric::Rain);
        let wind = norm(w.wind, Metric::Wind);
        let cloud = norm(w.cloud, Metric::Cloud);
        let sun = norm(w.sunlight, Metric::Sunlight);

        if rain > 0.35 && wind > 0.45 {
            Condition::Stormy
        } else if rain > 0.2 {
            Condition::Rainy
        } else if wind > 0.5 {
            Condition::Windy
        } else if cloud > 0.55 {
            Condition::Cloudy
        } else if sun > 0.45 && cloud < 0.35 {
            Condition::Sunny
        } else {
            Condition::Mixed
        }
    }

    fn rgba(self) -> Rgba {
        let (r, g, b) = match self {
            Condition::Sunny => (0xd4, 0xa0, 0x17),
            Condition::Cloudy => (0xc0, 0xc5, 0xce),
            Condition::Rainy => (0x3d, 0x7e, 0xaa),
            Condition::Windy => (0x8e, 0xb8, 0xb6),
            Condition::Stormy => (0x2a, 0x2d, 0x5e),
            Condition::Mixed => (0xa8, 0x9b, 0xc4),
        };
        Rgba { r, g, b, a: 255 }
    }
}

fn overlay_flags(w: &WeatherSample) -> u8 {
    let mut flags = 0u8;
    if norm(w.rain, Metric::Rain) > 0.35 && norm(w.wind, Metric::Wind) > 0.45 {
        flags |= FLAG_STORM;
    }
    if norm(w.sunlight, Metric::Sunlight) > 0.55 && norm(w.cloud, Metric::Cloud) < 0.3 {
        flags |= FLAG_BRIGHT;
    }
    flags
}

fn weather_segment(
    x: f32,
    y: f32,
    angle: f32,
    rgba: Rgba,
    w: &WeatherSample,
) -> SegmentDraw {
    SegmentDraw {
        pose: SegmentPose::from_angle(x, y, angle),
        rgba,
        rain: scale_u8(w.rain, Metric::Rain),
        wind: scale_u8(w.wind, Metric::Wind),
        temperature: scale_u8(w.temperature, Metric::Temperature),
        flags: overlay_flags(w),
    }
}

pub fn segment_count(mode: ViewMode, series: &MetricSeries) -> u32 {
    let hour_count = series.hour_count();
    match mode {
        ViewMode::Ribbon => hour_count.saturating_mul(4),
        ViewMode::Daylight => crate::daylight::count_daylight_hours(series),
        _ => hour_count,
    }
}

fn estimate_command_count(mode: ViewMode, segment_count: usize) -> usize {
    match mode {
        ViewMode::Metric | ViewMode::Ribbon => segment_count,
        ViewMode::Glyphs => segment_count * 10,
        ViewMode::Condition => segment_count * 4,
        ViewMode::Tapestry
        | ViewMode::Mandala
        | ViewMode::Fingerprint
        | ViewMode::Daylight => segment_count * 6 + 2,
    }
}

pub fn build_view_geometry(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    mode: ViewMode,
    metric: Metric,
    start_unix: u32,
    hour_step: u32,
    zoom: Zoom,
    canvas_size: u32,
) -> usize {
    let hour_count = series.hour_count() as usize;
    writer.reserve_commands(estimate_command_count(mode, hour_count));

    match mode {
        ViewMode::Metric => {
            build_metric(writer, series, metric, start_unix, hour_step, zoom, canvas_size);
            hour_count
        }
        ViewMode::Tapestry => {
            build_tapestry(writer, series, start_unix, hour_step, zoom, canvas_size);
            hour_count
        }
        ViewMode::Ribbon => build_ribbon(writer, series, start_unix, hour_step, zoom, canvas_size),
        ViewMode::Condition => {
            build_condition(writer, series, start_unix, hour_step, zoom, canvas_size);
            hour_count
        }
        ViewMode::Glyphs => {
            build_glyphs(writer, series, start_unix, hour_step, zoom, canvas_size);
            hour_count
        }
        ViewMode::Mandala => {
            build_mandala(writer, series, start_unix, hour_step, canvas_size);
            hour_count
        }
        ViewMode::Fingerprint => {
            build_fingerprint(writer, series, start_unix, hour_step, canvas_size);
            hour_count
        }
        ViewMode::Daylight => build_daylight(writer, series, canvas_size),
    }
}

fn build_metric(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    metric: Metric,
    start_unix: u32,
    hour_step: u32,
    zoom: Zoom,
    canvas_size: u32,
) {
    let hour_count = series.hour_count();
    let ctx = layout_ctx(hour_count, start_unix, hour_step, canvas_size, zoom);
    let (seg_w, seg_h) = layout_segment_size(canvas_size, zoom);
    let mut segments = Vec::with_capacity(hour_count as usize);

    for i in 0..hour_count {
        let value = read_f32_le(series.bytes(metric), i);
        let (x, y, angle) = layout_point(&ctx, hour_step, i);
        let rgba = colour_for(metric, value);
        segments.push(SegmentDraw {
            pose: SegmentPose::from_angle(x, y, angle),
            rgba,
            rain: 0,
            wind: 0,
            temperature: 0,
            flags: 0,
        });
    }
    push_sky_only(writer, seg_w, seg_h, &segments);
}

fn build_tapestry(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    start_unix: u32,
    hour_step: u32,
    zoom: Zoom,
    canvas_size: u32,
) {
    let hour_count = series.hour_count();
    let (seg_w, seg_h) = tapestry_segment_size(canvas_size, zoom);
    let ctx = tapestry_layout_ctx(hour_count, start_unix, hour_step, canvas_size, zoom);
    let mut segments = Vec::with_capacity(hour_count as usize);

    for i in 0..hour_count {
        let w = WeatherSample::from_series(series, i);
        let (x, y, angle) = layout_point(&ctx, hour_step, i);
        segments.push(weather_segment(x, y, angle, sky_rgba(w.sunlight, w.cloud), &w));
    }

    push_tapestry_layers(
        writer,
        ViewMode::Tapestry,
        f32_from_u32(canvas_size),
        f32_from_u32(canvas_size),
        seg_w,
        seg_h,
        &segments,
    );
}

fn build_ribbon(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    start_unix: u32,
    hour_step: u32,
    zoom: Zoom,
    canvas_size: u32,
) -> usize {
    let hour_count = series.hour_count();
    let ctx = layout_ctx(hour_count, start_unix, hour_step, canvas_size, zoom);
    let (seg_w, seg_h) = layout_segment_size(canvas_size, zoom);
    let track_w = seg_w * 0.22;
    let track_h = seg_h * 0.85;
    let metrics = [Metric::Sunlight, Metric::Cloud, Metric::Rain, Metric::Wind];
    let mut segments = Vec::with_capacity((hour_count * 4) as usize);

    for i in 0..hour_count {
        let (x, y, angle) = layout_point(&ctx, hour_step, i);
        for (track, &metric) in metrics.iter().enumerate() {
            let (ox, oy) = ribbon_track_offset(x, y, angle, f32_from_usize(track), seg_w);
            let value = read_f32_le(series.bytes(metric), i);
            let rgba = colour_for(metric, value);
            segments.push(SegmentDraw {
                pose: SegmentPose::from_angle(ox, oy, angle),
                rgba,
                rain: 0,
                wind: 0,
                temperature: 0,
                flags: 0,
            });
        }
    }
    push_sky_only(writer, track_w, track_h, &segments);
    (hour_count * 4) as usize
}

fn build_condition(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    start_unix: u32,
    hour_step: u32,
    zoom: Zoom,
    canvas_size: u32,
) {
    let hour_count = series.hour_count();
    let ctx = layout_ctx(hour_count, start_unix, hour_step, canvas_size, zoom);
    let (seg_w, seg_h) = layout_segment_size(canvas_size, zoom);
    let mut segments = Vec::with_capacity(hour_count as usize);

    for i in 0..hour_count {
        let w = WeatherSample::from_series(series, i);
        let condition = Condition::classify(&w);
        let (x, y, angle) = layout_point(&ctx, hour_step, i);
        let mut rgba = condition.rgba();
        let sun_t = norm(w.sunlight, Metric::Sunlight);
        let cloud_t = norm(w.cloud, Metric::Cloud);

        if condition == Condition::Stormy {
            rgba = Rgba {
                r: STORM_INDIGO.0,
                g: STORM_INDIGO.1,
                b: STORM_INDIGO.2,
                a: 255,
            };
        }

        let mut flags = overlay_flags(&w);
        if sun_t > 0.6 && cloud_t < 0.35 {
            flags |= FLAG_BRIGHT;
        }

        segments.push(SegmentDraw {
            pose: SegmentPose::from_angle(x, y, angle),
            rgba,
            rain: scale_u8(w.rain, Metric::Rain),
            wind: scale_u8(w.wind, Metric::Wind),
            temperature: scale_u8(w.temperature, Metric::Temperature),
            flags,
        });
    }
    push_condition_view(writer, seg_w, seg_h * 1.15, &segments);
}

fn build_glyphs(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    start_unix: u32,
    hour_step: u32,
    zoom: Zoom,
    canvas_size: u32,
) {
    let hour_count = series.hour_count();
    let ctx = layout_ctx(hour_count, start_unix, hour_step, canvas_size, zoom);
    let (seg_w, seg_h) = layout_segment_size(canvas_size, zoom);
    let mut segments = Vec::with_capacity(hour_count as usize);

    for i in 0..hour_count {
        let w = WeatherSample::from_series(series, i);
        let (x, y, angle) = layout_point(&ctx, hour_step, i);
        segments.push(weather_segment(x, y, angle, sky_rgba(w.sunlight, w.cloud), &w));
    }
    push_glyphs(writer, seg_w * 0.9, &segments);
    let _ = seg_h;
}

fn build_mandala(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    start_unix: u32,
    hour_step: u32,
    canvas_size: u32,
) {
    let hour_count = series.hour_count();
    let ctx = layout_ctx(hour_count, start_unix, hour_step, canvas_size, Zoom::Year);
    let layout = mandala_layout(canvas_size);
    let mut segments = Vec::with_capacity(hour_count as usize);

    for i in 0..hour_count {
        let w = WeatherSample::from_series(series, i);
        let t = ctx.extent_start + f32_from_u32(i) * f32_from_u32(hour_step);
        let (x, y, angle) = mandala_point(&layout, t, ctx.extent_start, hour_step);
        segments.push(weather_segment(x, y, angle, sky_rgba(w.sunlight, w.cloud), &w));
    }

    push_tapestry_layers(
        writer,
        ViewMode::Mandala,
        f32_from_u32(canvas_size),
        f32_from_u32(canvas_size),
        layout.seg_w,
        layout.seg_h,
        &segments,
    );
}

fn build_fingerprint(
    writer: &mut DrawWriter,
    series: &MetricSeries,
    start_unix: u32,
    hour_step: u32,
    canvas_size: u32,
) {
    let hour_count = series.hour_count();
    let ctx = layout_ctx(hour_count, start_unix, hour_step, canvas_size, Zoom::Year);
    let layout = fingerprint_layout(canvas_size);
    let mut segments = Vec::with_capacity(hour_count as usize);

    for i in 0..hour_count {
        let w = WeatherSample::from_series(series, i);
        let t = ctx.extent_start + f32_from_u32(i) * f32_from_u32(hour_step);
        let (x, y, angle) = fingerprint_point(&layout, t, ctx.extent_start, hour_step);
        segments.push(weather_segment(x, y, angle, sky_rgba(w.sunlight, w.cloud), &w));
    }

    push_tapestry_layers(
        writer,
        ViewMode::Fingerprint,
        f32_from_u32(canvas_size),
        f32_from_u32(canvas_size),
        layout.seg_w,
        layout.seg_h,
        &segments,
    );
}

fn build_daylight(writer: &mut DrawWriter, series: &MetricSeries, canvas_size: u32) -> usize {
    let hour_count = series.hour_count();
    let layout = daylight_layout(canvas_size);
    let mut segments = Vec::with_capacity(hour_count as usize / 2);

    for i in 0..hour_count {
        if !is_daylight_hour(series, i) {
            continue;
        }
        let day = i / 24;
        let hour_of_day = i % 24;
        let Some(window) = day_window(series, day) else {
            continue;
        };

        let w = WeatherSample::from_series(series, i);
        let (x, y, angle) = daylight_point(&layout, f32_from_u32(day), f32_from_u32(hour_of_day), window);
        segments.push(weather_segment(x, y, angle, sky_rgba(w.sunlight, w.cloud), &w));
    }

    push_tapestry_layers(
        writer,
        ViewMode::Daylight,
        f32_from_u32(canvas_size),
        f32_from_u32(canvas_size),
        layout.seg_w,
        layout.seg_h,
        &segments,
    );
    segments.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::DrawWriter;

    fn series_with_values(values: &[f32]) -> MetricSeries {
        let blob: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        MetricSeries::new(
            std::array::from_fn(|idx| if idx == 0 { blob.clone() } else { Vec::new() }),
            values.len().try_into().expect("test series length"),
        )
    }

    #[test]
    fn metric_view_emits_one_command_per_hour() {
        let series = series_with_values(&[0.0; 10]);
        let mut writer = DrawWriter::new();
        build_view_geometry(
            &mut writer,
            &series,
            ViewMode::Metric,
            Metric::Cloud,
            0,
            3600,
            Zoom::Year,
            750,
        );
        assert_eq!(writer.command_count(), 10);
    }

    fn full_series(values: &[f32]) -> MetricSeries {
        let blob: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        MetricSeries::new(
            std::array::from_fn(|_| blob.clone()),
            values.len().try_into().expect("test series length"),
        )
    }

    #[test]
    fn ribbon_quadruples_commands() {
        let series = full_series(&[10.0; 10]);
        let mut writer = DrawWriter::new();
        let written = build_view_geometry(
            &mut writer,
            &series,
            ViewMode::Ribbon,
            Metric::Cloud,
            0,
            3600,
            Zoom::Year,
            750,
        );
        assert_eq!(written, 40);
        assert_eq!(writer.command_count(), 40);
    }
}
