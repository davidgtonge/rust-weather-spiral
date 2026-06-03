use crate::state::Zoom;

#[derive(Debug, Clone, Copy)]
pub struct LayoutCtx {
    pub origin: f32,
    pub extent_start: f32,
    pub extent_end: f32,
    pub period: f32,
    pub inner_min: f32,
    pub inner_max: f32,
}

pub fn zoom_layout(zoom: Zoom) -> (u32, f32, f32, f32) {
    match zoom {
        Zoom::Year => (4, 220.0, 25.0, 1.0),
        Zoom::Month => (12, 160.0, 15.0, 4.0),
        Zoom::Week => (52, 200.0, 5.0, 15.0),
        Zoom::Day => (363, 0.0, 1.0, 20.0),
    }
}

pub fn layout_ctx(
    hour_count: u32,
    start_unix: u32,
    hour_step: u32,
    canvas_size: u32,
    zoom: Zoom,
) -> LayoutCtx {
    let (rotations, offset, _, _) = zoom_layout(zoom);
    let scale = canvas_size as f32 / 750.0;
    let origin = canvas_size as f32 / 2.0 - 30.0 * scale;
    let offset = offset * scale;

    let extent_start = start_unix as f32;
    let extent_end =
        start_unix as f32 + (hour_count.saturating_sub(1) as f32) * hour_step as f32;
    let period = (extent_end - extent_start) / rotations as f32;

    LayoutCtx {
        origin,
        extent_start,
        extent_end,
        period,
        inner_min: offset - 30.0 * scale,
        inner_max: origin - 30.0 * scale,
    }
}

#[inline]
pub fn layout_point(ctx: &LayoutCtx, hour_step: u32, index: u32) -> (f32, f32, f32) {
    let t = ctx.extent_start + index as f32 * hour_step as f32;
    let radius_t = (t - ctx.extent_start) / (ctx.extent_end - ctx.extent_start);
    let r = ctx.inner_min + radius_t * (ctx.inner_max - ctx.inner_min);
    let angle_t = (t - ctx.extent_start) / ctx.period;
    let angle = angle_t * std::f32::consts::TAU;
    (
        ctx.origin + r * angle.cos(),
        ctx.origin + r * angle.sin(),
        angle,
    )
}

pub fn layout_segment_size(canvas_size: u32, zoom: Zoom) -> (f32, f32) {
    let (_, _, segment_w, segment_h) = zoom_layout(zoom);
    let scale = canvas_size as f32 / 750.0;
    (segment_w * scale, segment_h * scale)
}

/// Tapestry segments are smaller than metric bricks — room for radial overlays on the spiral.
pub fn tapestry_segment_size(canvas_size: u32, zoom: Zoom) -> (f32, f32) {
    let (seg_w, seg_h) = layout_segment_size(canvas_size, zoom);
    const SCALE: f32 = 0.36;
    (seg_w * SCALE, seg_h * SCALE)
}

/// Canvas caps overlays when segment width is large (tapestry spiral).
pub const TAPESTRY_OVERLAY_EXTENT_CAP: f32 = 52.0;
pub const TAPESTRY_OVERLAY_GLOW_MAX: f32 = 34.0;

/// Daylight sun-glow cap (`spiral-canvas-modes.ts`).
const DAYLIGHT_GLOW_MAX: f32 = 10.0;

/// Small segments for dense ring views (mandala / fingerprint / daylight).
pub fn ring_segment_size(canvas_size: u32) -> (f32, f32) {
    let scale = canvas_size as f32 / 750.0;
    (5.5 * scale, 3.5 * scale)
}

/// Overlay scale for ring views — matches `overlayScale()` in `spiral-canvas-modes.ts`.
pub fn ring_overlay_scale(seg_w: f32, daylight: bool) -> (f32, f32) {
    let extent = 1.0;
    let glow = if daylight {
        (seg_w * 0.55).min(DAYLIGHT_GLOW_MAX)
    } else {
        seg_w * 0.55
    };
    (extent, glow)
}

/// Padding from canvas edge when the outer ring anchor is at `outer` radius from centre.
/// Glow is a circle around the anchor, so it must be counted separately from radial spokes.
pub fn ring_edge_pad(seg_w: f32, seg_h: f32, canvas_size: u32, daylight: bool) -> f32 {
    let scale = canvas_size as f32 / 750.0;
    let (extent, glow) = ring_overlay_scale(seg_w, daylight);
    let corner = (seg_w * seg_w + seg_h * seg_h).sqrt();
    let radial = seg_w * (0.5 + 1.4 * extent);
    corner + radial + glow + 4.0 * scale
}

/// Padding from canvas edge for anchor + rotated segment + capped overlays.
pub fn tapestry_edge_pad(seg_w: f32, seg_h: f32, canvas_size: u32) -> f32 {
    let scale = canvas_size as f32 / 750.0;
    let extent = (TAPESTRY_OVERLAY_EXTENT_CAP / seg_w.max(1.0)).min(1.0);
    let glow = (seg_w * 0.55).min(TAPESTRY_OVERLAY_GLOW_MAX);
    let corner = (seg_w * seg_w + seg_h * seg_h).sqrt();
    // Local +x is radial; rain extends to ~seg_w * (0.5 + 1.4 * extent) beyond anchor.
    corner + seg_w * (0.55 + 1.4 * extent) + glow + 12.0 * scale
}

/// Spiral layout for tapestry — centred with radial inset so segments + overlays fit.
pub fn tapestry_layout_ctx(
    hour_count: u32,
    start_unix: u32,
    hour_step: u32,
    canvas_size: u32,
    zoom: Zoom,
) -> LayoutCtx {
    let mut ctx = layout_ctx(hour_count, start_unix, hour_step, canvas_size, zoom);
    let (seg_w, seg_h) = tapestry_segment_size(canvas_size, zoom);
    let center = canvas_size as f32 * 0.5;
    let edge_pad = tapestry_edge_pad(seg_w, seg_h, canvas_size);
    let radial_span = (ctx.inner_max - ctx.inner_min).max(40.0 * (canvas_size as f32 / 750.0));
    let max_radius = (center - edge_pad).max(edge_pad);
    ctx.origin = center;
    ctx.inner_max = max_radius;
    ctx.inner_min = (max_radius - radial_span).max(edge_pad);
    ctx
}

/// Offset a spiral point perpendicular to the tangent for multi-track ribbon layouts.
pub fn ribbon_track_offset(x: f32, y: f32, angle: f32, track: f32, seg_w: f32) -> (f32, f32) {
    let spacing = seg_w * 0.28;
    let offset = (track - 1.5) * spacing;
    let px = -angle.sin();
    let py = angle.cos();
    (x + px * offset, y + py * offset)
}

/// Shared radial layout for dense calendar rings (mandala petals, fingerprint).
#[derive(Debug, Clone, Copy)]
pub struct RadialRingLayout {
    pub cx: f32,
    pub cy: f32,
    pub inner: f32,
    pub outer: f32,
    pub seg_w: f32,
    pub seg_h: f32,
}

fn radial_ring_layout(
    canvas_size: u32,
    inner_ratio: f32,
    daylight: bool,
    // Largest hour index that maps to the outer edge (23 for `hour / 24` layouts).
    max_hour_index: f32,
) -> RadialRingLayout {
    let cx = canvas_size as f32 * 0.5;
    let cy = canvas_size as f32 * 0.5;
    let (seg_w, seg_h) = ring_segment_size(canvas_size);
    let margin = ring_edge_pad(seg_w, seg_h, canvas_size, daylight);
    let max_anchor = cx.min(cy) - margin;
    // Scale outer so the highest-index hour actually reaches `max_anchor`.
    let outer = max_anchor * 24.0 / max_hour_index.max(1.0);
    let inner = outer * inner_ratio;
    RadialRingLayout {
        cx,
        cy,
        inner,
        outer,
        seg_w,
        seg_h,
    }
}

/// Layout for twelve-petal mandala — fits inside `canvas_size` with room for overlays.
#[derive(Debug, Clone, Copy)]
pub struct MandalaLayout {
    pub cx: f32,
    pub cy: f32,
    pub inner: f32,
    pub outer: f32,
    pub seg_w: f32,
    pub seg_h: f32,
}

pub fn mandala_layout(canvas_size: u32) -> MandalaLayout {
    let ring = radial_ring_layout(canvas_size, 0.14, false, 23.0);
    MandalaLayout {
        cx: ring.cx,
        cy: ring.cy,
        inner: ring.inner,
        outer: ring.outer,
        seg_w: ring.seg_w,
        seg_h: ring.seg_h,
    }
}

/// Hour-of-day radius × day-of-year angle — fits inside `canvas_size`.
pub type FingerprintLayout = RadialRingLayout;

pub fn fingerprint_layout(canvas_size: u32) -> FingerprintLayout {
    radial_ring_layout(canvas_size, 0.08, false, 23.0)
}

/// Sunprint ring — slightly wider band for daylight-only marks.
pub fn daylight_layout(canvas_size: u32) -> FingerprintLayout {
    // Lit-hour radius uses centred `t` within each day's window and can reach `outer`.
    radial_ring_layout(canvas_size, 0.12, true, 24.0)
}

/// Leap-year archive (8784 h).
pub const DAYS_PER_YEAR: f32 = 366.0;

/// Twelve-petal mandala: month sector + day/hour progression inside the petal.
pub fn mandala_point(
    layout: &MandalaLayout,
    unix_t: f32,
    start_unix: f32,
    hour_step: u32,
) -> (f32, f32, f32) {
    let hours_from_start = ((unix_t - start_unix) / hour_step as f32).max(0.0);
    let day = (hours_from_start / 24.0).floor();
    let hour_of_day = hours_from_start % 24.0;

    let month = ((day / 30.4).floor() as u32).min(11);
    let day_in_month = day % 30.4;
    let month_fraction = (day_in_month / 30.4).clamp(0.0, 0.999);

    let sector = std::f32::consts::TAU / 12.0;
    let hour_spread = (hour_of_day / 24.0) * sector * 0.04;
    let angle = month as f32 * sector + month_fraction * sector * 0.94 + hour_spread;
    let radius = layout.inner + (hour_of_day / 24.0) * (layout.outer - layout.inner);
    (
        layout.cx + radius * angle.cos(),
        layout.cy + radius * angle.sin(),
        angle,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Zoom;

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
        let outward_x = cos;
        let outward_y = sin;
        max_x += radial * outward_x.max(0.0);
        max_y += radial * outward_y.max(0.0);
        min_x += radial * outward_x.min(0.0);
        min_y += radial * outward_y.min(0.0);
        min_x -= glow;
        min_y -= glow;
        max_x += glow;
        max_y += glow;
        (min_x, min_y, max_x, max_y)
    }

    fn assert_ring_fills_canvas(outer: f32, canvas: u32) {
        let half = canvas as f32 * 0.5;
        const MIN_FILL: f32 = 0.95;
        assert!(
            outer / half >= MIN_FILL,
            "ring outer {outer} fills only {:.0}% of half-canvas (need {:.0}%)",
            outer / half * 100.0,
            MIN_FILL * 100.0
        );
    }

    fn assert_ring_point_fits(
        x: f32,
        y: f32,
        angle: f32,
        layout: &RadialRingLayout,
        canvas: u32,
        daylight: bool,
        label: &str,
    ) {
        let margin = 2.0_f32;
        let (cos, sin) = angle.sin_cos();
        let (min_x, min_y, max_x, max_y) = ring_overlay_bounds(
            x,
            y,
            cos,
            sin,
            layout.seg_w,
            layout.seg_h,
            daylight,
        );
        assert!(
            min_x >= margin
                && min_y >= margin
                && max_x <= canvas as f32 - margin
                && max_y <= canvas as f32 - margin,
            "{label} bbox [{min_x},{min_y}]-[{max_x},{max_y}] outside {canvas}"
        );
    }

    fn assert_tapestry_segments_fit(canvas: u32, hour_count: u32, start: u32, zoom: Zoom) {
        let hour_step = 3600u32;
        let ctx = tapestry_layout_ctx(hour_count, start, hour_step, canvas, zoom);
        let (seg_w, seg_h) = tapestry_segment_size(canvas, zoom);
        let margin = 2.0_f32;
        let extent = (TAPESTRY_OVERLAY_EXTENT_CAP / seg_w.max(1.0)).min(1.0);
        let glow = (seg_w * 0.55).min(TAPESTRY_OVERLAY_GLOW_MAX);
        let radial_overlay = seg_w * (0.55 + 1.4 * extent) + glow;
        let center = canvas as f32 * 0.5;

        for i in 0..hour_count {
            let (x, y, angle) = layout_point(&ctx, hour_step, i);
            let (cos, sin) = angle.sin_cos();
            let (min_x, min_y, max_x, max_y) =
                segment_world_bounds(x, y, cos, sin, seg_w, seg_h);

            // Overlays extend along local +x (radial outward from spiral centre).
            let outward_x = cos;
            let outward_y = sin;
            let max_xo = max_x + radial_overlay * outward_x.max(0.0);
            let max_yo = max_y + radial_overlay * outward_y.max(0.0);
            let min_xo = min_x + radial_overlay * outward_x.min(0.0);
            let min_yo = min_y + radial_overlay * outward_y.min(0.0);

            assert!(
                min_x >= margin
                    && min_y >= margin
                    && max_x <= canvas as f32 - margin
                    && max_y <= canvas as f32 - margin,
                "tapestry segment {i} bbox [{min_x},{min_y}]-[{max_x},{max_y}] outside {canvas} (zoom {zoom:?})"
            );
            assert!(
                min_xo >= margin
                    && min_yo >= margin
                    && max_xo <= canvas as f32 - margin
                    && max_yo <= canvas as f32 - margin,
                "tapestry segment {i} with overlays outside {canvas} (zoom {zoom:?}, centre {center})"
            );
        }
    }

    #[test]
    fn tapestry_spiral_stays_inside_canvas() {
        let canvas = 1024u32;
        let hour_count = 8784u32;
        let ljubljana_start = 1_704_063_600u32;
        for zoom in Zoom::ALL {
            assert_tapestry_segments_fit(canvas, hour_count, ljubljana_start, zoom);
        }
    }

    #[test]
    fn fingerprint_ring_fills_canvas_and_overlays_fit() {
        let canvas = 1024u32;
        let layout = fingerprint_layout(canvas);
        assert_ring_fills_canvas(layout.outer, canvas);

        let hour_step = 3600u32;
        let start = 1_704_067_200u32;
        let hour_count = 8784u32;

        for i in 0..hour_count {
            let t = start as f32 + i as f32 * hour_step as f32;
            let (x, y, angle) = fingerprint_point(&layout, t, start as f32, hour_step);
            assert_ring_point_fits(
                x,
                y,
                angle,
                &layout,
                canvas,
                false,
                &format!("fingerprint hour {i}"),
            );
        }
    }

    #[test]
    fn mandala_ring_fills_canvas_and_overlays_fit() {
        let canvas = 1024u32;
        let layout = mandala_layout(canvas);
        assert_ring_fills_canvas(layout.outer, canvas);

        let hour_step = 3600u32;
        let start = 1_704_067_200u32;
        let hour_count = 8784u32;

        for i in 0..hour_count {
            let t = start as f32 + i as f32 * hour_step as f32;
            let (x, y, angle) = mandala_point(&layout, t, start as f32, hour_step);
            assert_ring_point_fits(
                x,
                y,
                angle,
                &RadialRingLayout {
                    cx: layout.cx,
                    cy: layout.cy,
                    inner: layout.inner,
                    outer: layout.outer,
                    seg_w: layout.seg_w,
                    seg_h: layout.seg_h,
                },
                canvas,
                false,
                &format!("mandala hour {i}"),
            );
        }

        assert!(layout.outer > layout.inner);
    }
}

/// Sunprint: angle = day of year, radius = position within that day's lit hours only.
pub fn daylight_point(
    layout: &FingerprintLayout,
    day: f32,
    hour_of_day: f32,
    window: crate::daylight::DayWindow,
) -> (f32, f32, f32) {
    let angle = (day / DAYS_PER_YEAR) * std::f32::consts::TAU;
    let span = (window.last_hour - window.first_hour + 1) as f32;
    let t = ((hour_of_day - window.first_hour as f32) + 0.5) / span;
    let radius = layout.inner + t.clamp(0.0, 1.0) * (layout.outer - layout.inner);
    (
        layout.cx + radius * angle.cos(),
        layout.cy + radius * angle.sin(),
        angle,
    )
}

/// Circular year calendar: angle = day of year, radius = hour of day.
pub fn fingerprint_point(
    layout: &FingerprintLayout,
    unix_t: f32,
    start_unix: f32,
    hour_step: u32,
) -> (f32, f32, f32) {
    let hours_from_start = ((unix_t - start_unix) / hour_step as f32).max(0.0);
    let day = (hours_from_start / 24.0).floor();
    let hour_of_day = hours_from_start % 24.0;
    let angle = (day / DAYS_PER_YEAR) * std::f32::consts::TAU;
    let radius = layout.inner + (hour_of_day / 24.0) * (layout.outer - layout.inner);
    (
        layout.cx + radius * angle.cos(),
        layout.cy + radius * angle.sin(),
        angle,
    )
}

