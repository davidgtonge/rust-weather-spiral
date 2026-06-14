//! Canvas draw-command wire — Rust owns layout and overlay geometry; TS executes ops.

use crate::colour::Rgba;
use crate::num::{u32_from_usize, u8_from_alpha, u8_from_f32_rounded};
use crate::spiral::{ring_overlay_scale, TAPESTRY_OVERLAY_EXTENT_CAP, TAPESTRY_OVERLAY_GLOW_MAX};
use crate::view_mode::ViewMode;

/// Wire header: `u32 count`, `f32 canvas_w`, `f32 canvas_h`, `u32 playhead`, `u32 view_mode`.
pub const HEADER_LEN: usize = 20;
pub const CMD_BYTES: usize = 48;

pub const OP_FILL_RECT: u8 = 0;
pub const OP_STROKE_LINE: u8 = 1;
pub const OP_STROKE_ARC: u8 = 2;
pub const OP_FILL_CIRCLE: u8 = 3;
pub const OP_FILL_ELLIPSE: u8 = 4;
pub const OP_STROKE_RECT: u8 = 5;
pub const OP_BEGIN_CLIP: u8 = 6;
pub const OP_END_CLIP: u8 = 7;

pub const FL_CANVAS_SPACE: u8 = 1;

const RAIN_MARK_MIN: u8 = 8;
const WIND_MARK_MIN: u8 = 10;
const GLYPH_RAIN_MIN: u8 = 12;
const GLYPH_WIND_MIN: u8 = 12;

pub fn wire_byte_len(command_count: u32) -> usize {
    HEADER_LEN + command_count as usize * CMD_BYTES
}

pub struct DrawWriter {
    out: Vec<u8>,
}

impl DrawWriter {
    pub fn new() -> Self {
        Self {
            out: vec![0u8; HEADER_LEN],
        }
    }

    pub fn reserve_commands(&mut self, n: usize) {
        self.out.reserve(n * CMD_BYTES);
    }

    pub fn command_count(&self) -> usize {
        (self.out.len().saturating_sub(HEADER_LEN)) / CMD_BYTES
    }

    pub fn push_cmd(
        &mut self,
        op: u8,
        flags: u8,
        rgba: Rgba,
        line_width: f32,
        cos: f32,
        sin: f32,
        tx: f32,
        ty: f32,
        p0: f32,
        p1: f32,
        p2: f32,
        p3: f32,
        p4: f32,
    ) {
        self.out.push(op);
        self.out.push(flags);
        self.out.push(rgba.r);
        self.out.push(rgba.g);
        self.out.push(rgba.b);
        self.out.push(rgba.a);
        self.out.push(0);
        self.out.push(0);
        self.out.extend_from_slice(&line_width.to_le_bytes());
        self.out.extend_from_slice(&cos.to_le_bytes());
        self.out.extend_from_slice(&sin.to_le_bytes());
        self.out.extend_from_slice(&tx.to_le_bytes());
        self.out.extend_from_slice(&ty.to_le_bytes());
        self.out.extend_from_slice(&p0.to_le_bytes());
        self.out.extend_from_slice(&p1.to_le_bytes());
        self.out.extend_from_slice(&p2.to_le_bytes());
        self.out.extend_from_slice(&p3.to_le_bytes());
        self.out.extend_from_slice(&p4.to_le_bytes());
    }

    pub fn finish(
        mut self,
        view_mode: ViewMode,
        playhead_index: u32,
        canvas_w: f32,
        canvas_h: f32,
    ) -> Vec<u8> {
        let count = u32_from_usize(self.command_count());
        self.out[0..4].copy_from_slice(&count.to_le_bytes());
        self.out[4..8].copy_from_slice(&canvas_w.to_le_bytes());
        self.out[8..12].copy_from_slice(&canvas_h.to_le_bytes());
        self.out[12..16].copy_from_slice(&playhead_index.to_le_bytes());
        self.out[16..20].copy_from_slice(&view_mode.as_u32().to_le_bytes());
        self.out
    }
}

#[derive(Clone, Copy)]
pub struct SegmentPose {
    pub x: f32,
    pub y: f32,
    pub cos: f32,
    pub sin: f32,
}

impl SegmentPose {
    pub fn from_angle(x: f32, y: f32, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self { x, y, cos, sin }
    }
}

fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba { r, g, b, a }
}

fn rgba_f32(r: u8, g: u8, b: u8, alpha: f32) -> Rgba {
    Rgba {
        r,
        g,
        b,
        a: u8_from_alpha(alpha),
    }
}

fn lerp_u8(t: f32, a: u8, b: u8) -> u8 {
    let af = f32::from(a);
    let bf = f32::from(b);
    u8_from_f32_rounded(af + (bf - af) * t)
}

/// Glyph temperature strip — matches former `temperatureRgb()` in TS.
pub fn temperature_glyph_rgb(temperature_u8: u8) -> Rgba {
    let t = f32::from(temperature_u8) / 255.0;
    let (r, g, b) = if t < 0.5 {
        let u = t * 2.0;
        (
            lerp_u8(u, 0x3b, 0xf7),
            lerp_u8(u, 0x4c, 0xf7),
            lerp_u8(u, 0xc0, 0xf7),
        )
    } else {
        let u = (t - 0.5) * 2.0;
        (
            lerp_u8(u, 0xf7, 0xb4),
            lerp_u8(u, 0xf7, 0x04),
            lerp_u8(u, 0xf7, 0x26),
        )
    };
    rgba_u8(r, g, b, 255)
}

fn tapestry_overlay_scale(seg_w: f32) -> (f32, f32) {
    let extent = (TAPESTRY_OVERLAY_EXTENT_CAP / seg_w.max(1.0)).min(1.0);
    let glow = (seg_w * 0.55).min(TAPESTRY_OVERLAY_GLOW_MAX);
    (extent, glow)
}

fn overlay_scale(view_mode: ViewMode, seg_w: f32) -> (f32, f32) {
    match view_mode {
        ViewMode::Daylight => ring_overlay_scale(seg_w, true),
        ViewMode::Mandala | ViewMode::Fingerprint => ring_overlay_scale(seg_w, false),
        ViewMode::Tapestry | ViewMode::Metric | ViewMode::Ribbon | ViewMode::Condition
        | ViewMode::Glyphs => tapestry_overlay_scale(seg_w),
    }
}

pub fn push_clip_rect(writer: &mut DrawWriter, pad: f32, canvas_w: f32, canvas_h: f32) {
    writer.push_cmd(
        OP_BEGIN_CLIP,
        0,
        rgba_u8(0, 0, 0, 255),
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        pad,
        pad,
        canvas_w - pad * 2.0,
        canvas_h - pad * 2.0,
        0.0,
    );
}

pub fn push_end_clip(writer: &mut DrawWriter) {
    writer.push_cmd(
        OP_END_CLIP,
        0,
        rgba_u8(0, 0, 0, 255),
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
}

pub fn push_fill_rect_local(
    writer: &mut DrawWriter,
    pose: SegmentPose,
    rgba: Rgba,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    writer.push_cmd(
        OP_FILL_RECT,
        0,
        rgba,
        0.0,
        pose.cos,
        pose.sin,
        pose.x,
        pose.y,
        x,
        y,
        w,
        h,
        0.0,
    );
}

pub fn push_stroke_line_local(
    writer: &mut DrawWriter,
    pose: SegmentPose,
    rgba: Rgba,
    line_width: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) {
    writer.push_cmd(
        OP_STROKE_LINE,
        0,
        rgba,
        line_width,
        pose.cos,
        pose.sin,
        pose.x,
        pose.y,
        x1,
        y1,
        x2,
        y2,
        0.0,
    );
}

pub fn push_stroke_arc_local(
    writer: &mut DrawWriter,
    pose: SegmentPose,
    rgba: Rgba,
    line_width: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    start: f32,
    end: f32,
) {
    writer.push_cmd(
        OP_STROKE_ARC,
        0,
        rgba,
        line_width,
        pose.cos,
        pose.sin,
        pose.x,
        pose.y,
        cx,
        cy,
        radius,
        start,
        end,
    );
}

pub fn push_fill_circle_canvas(
    writer: &mut DrawWriter,
    rgba: Rgba,
    cx: f32,
    cy: f32,
    radius: f32,
) {
    writer.push_cmd(
        OP_FILL_CIRCLE,
        FL_CANVAS_SPACE,
        rgba,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        cx,
        cy,
        radius,
        0.0,
        0.0,
    );
}

pub fn push_fill_circle_local(
    writer: &mut DrawWriter,
    pose: SegmentPose,
    rgba: Rgba,
    cx: f32,
    cy: f32,
    radius: f32,
) {
    writer.push_cmd(
        OP_FILL_CIRCLE,
        0,
        rgba,
        0.0,
        pose.cos,
        pose.sin,
        pose.x,
        pose.y,
        cx,
        cy,
        radius,
        0.0,
        0.0,
    );
}

pub fn push_fill_ellipse_local(
    writer: &mut DrawWriter,
    pose: SegmentPose,
    rgba: Rgba,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
) {
    writer.push_cmd(
        OP_FILL_ELLIPSE,
        0,
        rgba,
        0.0,
        pose.cos,
        pose.sin,
        pose.x,
        pose.y,
        cx,
        cy,
        rx,
        ry,
        0.0,
    );
}

pub fn push_stroke_rect_local(
    writer: &mut DrawWriter,
    pose: SegmentPose,
    rgba: Rgba,
    line_width: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    writer.push_cmd(
        OP_STROKE_RECT,
        0,
        rgba,
        line_width,
        pose.cos,
        pose.sin,
        pose.x,
        pose.y,
        x,
        y,
        w,
        h,
        0.0,
    );
}

pub struct SegmentDraw {
    pub pose: SegmentPose,
    pub rgba: Rgba,
    pub rain: u8,
    pub wind: u8,
    pub temperature: u8,
    pub flags: u8,
}

pub const FLAG_STORM: u8 = 1;
pub const FLAG_BRIGHT: u8 = 2;

pub fn push_sky_segment(writer: &mut DrawWriter, seg_w: f32, seg_h: f32, seg: &SegmentDraw) {
    push_fill_rect_local(writer, seg.pose, seg.rgba, 0.0, 0.0, seg_w, seg_h);
}

pub fn push_rain_mark(
    writer: &mut DrawWriter,
    seg_w: f32,
    seg: &SegmentDraw,
    extent: f32,
) {
    if seg.rain < RAIN_MARK_MIN {
        return;
    }
    let intensity = f32::from(seg.rain);
    let len = (intensity / 255.0) * seg_w * 1.4 * extent;
    let width = if seg.rain > 180 { 2.2 } else { 1.2 };
    push_stroke_line_local(
        writer,
        seg.pose,
        rgba_f32(45, 106, 159, 0.85),
        width,
        seg_w * 0.5,
        seg_w * 0.1,
        seg_w * 0.5 + len,
        seg_w * 0.1,
    );
}

pub fn push_wind_whisker(
    writer: &mut DrawWriter,
    seg_w: f32,
    seg: &SegmentDraw,
    extent: f32,
) {
    if seg.wind < WIND_MARK_MIN {
        return;
    }
    let intensity = f32::from(seg.wind);
    let len = (intensity / 255.0) * seg_w * 1.8 * extent;
    let width = if seg.wind > 200 { 1.8 } else { 0.9 };
    push_stroke_line_local(
        writer,
        seg.pose,
        rgba_f32(200, 210, 225, 0.7),
        width,
        0.0,
        seg_w * 0.35,
        len,
        seg_w * 0.35,
    );
}

pub fn push_storm_emphasis(
    writer: &mut DrawWriter,
    seg_w: f32,
    seg_h: f32,
    seg: &SegmentDraw,
    extent: f32,
) {
    if seg.flags & FLAG_STORM == 0 {
        return;
    }
    push_fill_rect_local(
        writer,
        seg.pose,
        rgba_f32(26, 31, 74, 0.55 * 0.75),
        -seg_w * 0.05 * extent,
        -seg_h * 0.1,
        seg_w * 1.1 * extent,
        seg_h * 1.2,
    );
}

pub fn push_sun_glow(writer: &mut DrawWriter, seg: &SegmentDraw, glow_radius: f32) {
    if seg.flags & FLAG_BRIGHT == 0 {
        return;
    }
    push_fill_circle_canvas(
        writer,
        rgba_f32(245, 200, 66, 0.22),
        seg.pose.x,
        seg.pose.y,
        glow_radius,
    );
}

pub fn push_condition_rain_bar(
    writer: &mut DrawWriter,
    seg_w: f32,
    seg_h: f32,
    seg: &SegmentDraw,
) {
    let rain_t = f32::from(seg.rain) / 255.0;
    let wind_t = f32::from(seg.wind) / 255.0;
    if rain_t < 0.15 && wind_t < 0.15 {
        return;
    }
    let extra_h = seg_h * (0.2 + rain_t * 0.8);
    push_fill_rect_local(
        writer,
        seg.pose,
        rgba_f32(45, 106, 159, 0.35 + rain_t * 0.4),
        0.0,
        0.0,
        seg_w,
        extra_h,
    );
}

pub fn push_glyph(writer: &mut DrawWriter, seg_w: f32, seg: &SegmentDraw) {
    let body_w = seg_w * 0.62;
    let body_h = seg_w * 0.58;
    let half_h = body_h * 0.5;
    let top_y = -half_h;
    let bottom_y = 0.0;
    let temp_rgba = temperature_glyph_rgb(seg.temperature);

    push_fill_rect_local(
        writer,
        seg.pose,
        seg.rgba,
        -body_w * 0.5,
        top_y,
        body_w,
        body_h * 0.52,
    );
    push_fill_rect_local(
        writer,
        seg.pose,
        temp_rgba,
        -body_w * 0.5,
        bottom_y,
        body_w,
        body_h * 0.48,
    );
    push_fill_rect_local(
        writer,
        seg.pose,
        rgba_f32(10, 12, 16, 0.35),
        -body_w * 0.5,
        -0.5,
        body_w,
        1.0,
    );

    let cloud_arc = ((255.0 - f32::from(seg.rgba.r)) / 180.0).min(1.0);
    if cloud_arc > 0.15 {
        push_stroke_arc_local(
            writer,
            seg.pose,
            rgba_f32(220, 225, 235, 0.35 + cloud_arc * 0.45),
            1.2,
            0.0,
            top_y + body_h * 0.12,
            body_w * 0.35,
            std::f32::consts::PI,
            0.0,
        );
    }

    if seg.rain > GLYPH_RAIN_MIN {
        let rain_val = f32::from(seg.rain);
        let drop = (rain_val / 255.0) * body_h * 0.5;
        push_fill_ellipse_local(
            writer,
            seg.pose,
            rgba_f32(45, 106, 159, 0.45 + rain_val / 512.0),
            0.0,
            top_y + body_h * 0.36,
            body_w * 0.12,
            drop * 0.45,
        );
    }

    if seg.wind > GLYPH_WIND_MIN {
        let wind_val = f32::from(seg.wind);
        let slash = (wind_val / 255.0) * body_w * 0.8;
        let alpha = 0.4 + wind_val / 400.0;
        push_stroke_line_local(
            writer,
            seg.pose,
            rgba_f32(200, 210, 225, alpha),
            1.0,
            -body_w * 0.2,
            bottom_y + body_h * 0.18,
            -body_w * 0.2 + slash,
            bottom_y + body_h * 0.18,
        );
        push_stroke_line_local(
            writer,
            seg.pose,
            rgba_f32(200, 210, 225, alpha),
            1.0,
            -body_w * 0.28,
            bottom_y + body_h * 0.28,
            -body_w * 0.28 + slash * 0.78,
            bottom_y + body_h * 0.28,
        );
    }

    if seg.flags & FLAG_BRIGHT != 0 {
        push_fill_circle_local(
            writer,
            seg.pose,
            rgba_f32(245, 200, 66, 0.92),
            body_w * 0.22,
            top_y + body_h * 0.12,
            body_w * 0.08,
        );
    }

    if seg.flags & FLAG_STORM != 0 {
        push_stroke_rect_local(
            writer,
            seg.pose,
            rgba_f32(26, 31, 74, 0.95),
            1.3,
            -body_w * 0.5,
            top_y,
            body_w,
            body_h,
        );
    }
}

pub fn push_tapestry_layers(
    writer: &mut DrawWriter,
    view_mode: ViewMode,
    canvas_w: f32,
    canvas_h: f32,
    seg_w: f32,
    seg_h: f32,
    segments: &[SegmentDraw],
) {
    let (extent, glow) = overlay_scale(view_mode, seg_w);
    push_clip_rect(writer, 2.0, canvas_w, canvas_h);

    for seg in segments.iter().rev() {
        push_sky_segment(writer, seg_w, seg_h, seg);
    }
    for seg in segments {
        push_rain_mark(writer, seg_w, seg, extent);
    }
    for seg in segments {
        push_wind_whisker(writer, seg_w, seg, extent);
    }
    for seg in segments {
        push_storm_emphasis(writer, seg_w, seg_h, seg, extent);
    }
    for seg in segments {
        push_sun_glow(writer, seg, glow);
    }

    push_end_clip(writer);
}

pub fn push_condition_view(
    writer: &mut DrawWriter,
    seg_w: f32,
    seg_h: f32,
    segments: &[SegmentDraw],
) {
    let (extent, _) = tapestry_overlay_scale(seg_w);
    for seg in segments.iter().rev() {
        push_sky_segment(writer, seg_w, seg_h, seg);
    }
    for seg in segments {
        push_rain_mark(writer, seg_w, seg, extent);
    }
    for seg in segments {
        push_wind_whisker(writer, seg_w, seg, extent);
    }
    for seg in segments {
        push_condition_rain_bar(writer, seg_w, seg_h, seg);
    }
}

pub fn push_glyphs(writer: &mut DrawWriter, seg_w: f32, segments: &[SegmentDraw]) {
    for seg in segments.iter().rev() {
        push_glyph(writer, seg_w, seg);
    }
}

pub fn push_sky_only(
    writer: &mut DrawWriter,
    seg_w: f32,
    seg_h: f32,
    segments: &[SegmentDraw],
) {
    for seg in segments.iter().rev() {
        push_sky_segment(writer, seg_w, seg_h, seg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_glow_uses_canvas_space() {
        let mut writer = DrawWriter::new();
        let seg = SegmentDraw {
            pose: SegmentPose {
                x: 200.0,
                y: 300.0,
                cos: 1.0,
                sin: 0.0,
            },
            rgba: rgba_u8(245, 200, 66, 255),
            rain: 0,
            wind: 0,
            temperature: 128,
            flags: FLAG_BRIGHT,
        };
        push_sun_glow(&mut writer, &seg, 8.0);
        assert_eq!(writer.command_count(), 1);
        let base = HEADER_LEN;
        assert_eq!(writer.out[base], OP_FILL_CIRCLE);
        assert_eq!(writer.out[base + 1], FL_CANVAS_SPACE);
        // f32 block starts at base+8: line_width, cos, sin, tx, ty, p0, p1, ...
        let cx = f32::from_le_bytes(writer.out[base + 28..base + 32].try_into().unwrap());
        let cy = f32::from_le_bytes(writer.out[base + 32..base + 36].try_into().unwrap());
        assert!((cx - 200.0).abs() < 1e-4, "cx={cx}");
        assert!((cy - 300.0).abs() < 1e-4, "cy={cy}");
    }

    #[test]
    fn tapestry_overlay_scale_caps_extent() {
        let seg_w = 8.0;
        let (extent, glow) = tapestry_overlay_scale(seg_w);
        let radial = seg_w * (0.55 + 1.4 * extent) + glow;
        assert!(radial > seg_w);
    }
}
