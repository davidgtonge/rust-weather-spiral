//! Compact draw-command wire for Canvas2D — Rust owns geometry; TS executes ops.

use crate::draw::{DrawWriter, wire_byte_len as draw_wire_byte_len, CMD_BYTES};
use crate::state::{Metric, MetricSeries, Zoom};
use crate::view_mode::ViewMode;
use crate::views::{build_view_geometry, segment_count};

/// One draw command per wire entry (legacy name kept for TS contract docs).
pub const BYTES_PER_SEGMENT: usize = CMD_BYTES;

pub fn wire_byte_len(command_count: u32) -> usize {
    draw_wire_byte_len(command_count)
}

#[allow(dead_code)]
pub const GEOMETRY_HEADER: usize = crate::draw::HEADER_LEN;

pub fn build_geometry_wire(
    series: &MetricSeries,
    metric: Metric,
    mode: ViewMode,
    start_unix: u32,
    hour_step: u32,
    zoom: Zoom,
    canvas_size: u32,
    playhead_index: u32,
) -> Vec<u8> {
    let mut writer = DrawWriter::new();
    let _segment_count = segment_count(mode, series);
    let _written = build_view_geometry(
        &mut writer,
        series,
        mode,
        metric,
        start_unix,
        hour_step,
        zoom,
        canvas_size,
    );
    writer.finish(
        mode,
        playhead_index,
        canvas_size as f32,
        canvas_size as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::load_embedded_cities;
    use crate::draw::{DrawWriter, FL_CANVAS_SPACE, HEADER_LEN, OP_FILL_CIRCLE};
    use crate::spiral::mandala_layout;
    use crate::spiral::{daylight_layout, daylight_point, ring_edge_pad, ring_overlay_scale};
    use crate::state::{Metric, MetricSeries, Zoom, FRAME_SIZE};
    use crate::view_mode::ViewMode;
    use crate::daylight::{day_window, is_daylight_hour};

    fn series_with_values(values: &[f32]) -> MetricSeries {
        let blob: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        MetricSeries::new(
            std::array::from_fn(|idx| if idx == 0 { blob.clone() } else { Vec::new() }),
            values.len() as u32,
        )
    }

    fn full_series(values: &[f32]) -> MetricSeries {
        let blob: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        MetricSeries::new(std::array::from_fn(|_| blob.clone()), values.len() as u32)
    }

    #[test]
    fn geometry_wire_size_matches_command_count() {
        let series = series_with_values(&(0..100).map(|i| i as f32).collect::<Vec<_>>());
        let wire = build_geometry_wire(
            &series,
            Metric::Cloud,
            ViewMode::Metric,
            0,
            3600,
            Zoom::Year,
            750,
            0,
        );
        let count = u32::from_le_bytes(wire[0..4].try_into().unwrap());
        assert_eq!(wire.len(), wire_byte_len(count));
        assert_eq!(count as usize, 100);
    }

    #[test]
    fn header_carries_canvas_and_view_mode() {
        let series = series_with_values(&[0.0, 1.0]);
        let wire = build_geometry_wire(
            &series,
            Metric::Cloud,
            ViewMode::Metric,
            0,
            3600,
            Zoom::Year,
            750,
            1,
        );
        assert_eq!(f32::from_le_bytes(wire[4..8].try_into().unwrap()), 750.0);
        assert_eq!(u32::from_le_bytes(wire[12..16].try_into().unwrap()), 1);
        assert_eq!(
            u32::from_le_bytes(wire[16..20].try_into().unwrap()),
            ViewMode::Metric.as_u32()
        );
    }

    fn segment_world_bounds(
        px: f32,
        py: f32,
        cos: f32,
        sin: f32,
        seg_w: f32,
        seg_h: f32,
    ) -> (f32, f32, f32, f32) {
        let corners = [(0.0, 0.0), (seg_w, 0.0), (seg_w, seg_h), (0.0, seg_h)];
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for (lx, ly) in corners {
            let wx = px + cos * lx - sin * ly;
            let wy = py + sin * lx + cos * ly;
            min_x = min_x.min(wx);
            min_y = min_y.min(wy);
            max_x = max_x.max(wx);
            max_y = max_y.max(wy);
        }
        (min_x, min_y, max_x, max_y)
    }

    fn ring_overlay_bounds(
        px: f32,
        py: f32,
        cos: f32,
        sin: f32,
        seg_w: f32,
        seg_h: f32,
        daylight: bool,
    ) -> (f32, f32, f32, f32) {
        let (extent, glow) = ring_overlay_scale(seg_w, daylight);
        let radial = seg_w * (0.5 + 1.4 * extent);
        let (mut min_x, mut min_y, mut max_x, mut max_y) =
            segment_world_bounds(px, py, cos, sin, seg_w, seg_h);
        max_x += radial * cos.max(0.0);
        max_y += radial * sin.max(0.0);
        min_x += radial * cos.min(0.0);
        min_y += radial * sin.min(0.0);
        min_x -= glow;
        min_y -= glow;
        max_x += glow;
        max_y += glow;
        (min_x, min_y, max_x, max_y)
    }

    #[test]
    fn daylight_geometry_in_bounds_all_cities() {
        let canvas = FRAME_SIZE;
        let cities = load_embedded_cities().expect("cities");
        let layout = daylight_layout(canvas);
        let margin = 2.0_f32;
        let half = canvas as f32 * 0.5;
        let max_anchor = half - ring_edge_pad(layout.seg_w, layout.seg_h, canvas, true);
        assert!(
            max_anchor / half >= 0.93,
            "daylight ring max anchor {max_anchor} should fill canvas (>{:.0}% of half)",
            93.0
        );

        for city in &cities {
            let wire = build_geometry_wire(
                &city.metrics,
                Metric::Cloud,
                ViewMode::Daylight,
                city.start_unix,
                city.hour_step,
                Zoom::Year,
                canvas,
                0,
            );
            let _count = u32::from_le_bytes(wire[0..4].try_into().unwrap()) as usize;

            let mut written = 0usize;
            for i in 0..city.metrics.hour_count() {
                if !is_daylight_hour(&city.metrics, i) {
                    continue;
                }
                let day = i / 24;
                let hour_of_day = i % 24;
                let Some(window) = day_window(&city.metrics, day) else {
                    continue;
                };
                let (x, y, angle) = daylight_point(
                    &layout,
                    day as f32,
                    hour_of_day as f32,
                    window,
                );
                let (cos, sin) = angle.sin_cos();
                let (min_x, min_y, max_x, max_y) = ring_overlay_bounds(
                    x,
                    y,
                    cos,
                    sin,
                    layout.seg_w,
                    layout.seg_h,
                    true,
                );
                assert!(
                    min_x >= margin
                        && min_y >= margin
                        && max_x <= canvas as f32 - margin
                        && max_y <= canvas as f32 - margin,
                    "{} segment {} bbox [{min_x},{min_y}]-[{max_x},{max_y}]",
                    city.id,
                    i
                );
                written += 1;
            }
            assert!(written > 0);
        }
    }

    #[test]
    fn mandala_wire_spans_all_quadrants() {
        let cities = load_embedded_cities().expect("cities");
        let city = cities.first().expect("city");
        let canvas = FRAME_SIZE;
        let layout = mandala_layout(canvas);
        let center = canvas as f32 * 0.5;
        let mut quads = [0usize; 4];

        for i in 0..city.metrics.hour_count() {
            let ctx = crate::spiral::layout_ctx(
                city.metrics.hour_count(),
                city.start_unix,
                city.hour_step,
                canvas,
                Zoom::Year,
            );
            let t = ctx.extent_start + i as f32 * city.hour_step as f32;
            let (x, y, _) = crate::spiral::mandala_point(&layout, t, ctx.extent_start, city.hour_step);
            let quad = usize::from(x >= center) + usize::from(y < center) * 2;
            quads[quad] += 1;
        }
        for (qi, &n) in quads.iter().enumerate() {
            assert!(n > 500, "quadrant {qi} has only {n} points");
        }
    }

    #[test]
    fn ribbon_wire_has_four_commands_per_hour() {
        let series = full_series(&[10.0; 10]);
        let wire = build_geometry_wire(
            &series,
            Metric::Cloud,
            ViewMode::Ribbon,
            0,
            3600,
            Zoom::Year,
            750,
            0,
        );
        let count = u32::from_le_bytes(wire[0..4].try_into().unwrap());
        assert_eq!(count, 40);
    }

    #[test]
    fn bright_fixture_emits_canvas_space_glow() {
        let mut writer = DrawWriter::new();
        crate::draw::push_sun_glow(
            &mut writer,
            &crate::draw::SegmentDraw {
                pose: crate::draw::SegmentPose {
                    x: 200.0,
                    y: 300.0,
                    cos: 1.0,
                    sin: 0.0,
                },
                rgba: crate::colour::Rgba {
                    r: 245,
                    g: 200,
                    b: 66,
                    a: 255,
                },
                rain: 0,
                wind: 0,
                temperature: 128,
                flags: crate::draw::FLAG_BRIGHT,
            },
            8.0,
        );
        let wire = writer.finish(ViewMode::Mandala, 0, 1024.0, 1024.0);
        let base = HEADER_LEN;
        assert_eq!(wire[base], OP_FILL_CIRCLE);
        assert_eq!(wire[base + 1], FL_CANVAS_SPACE);
    }
}
