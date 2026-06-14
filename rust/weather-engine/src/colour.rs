use crate::num::u8_from_f32_rounded;
use crate::state::Metric;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub fn metric_domain(metric: Metric) -> (f32, f32) {
    match metric {
        Metric::Cloud => (0.0, 100.0),
        Metric::Sunlight => (0.0, 800.0),
        Metric::Rain => (0.0, 10.0),
        Metric::Wind => (0.0, 25.0),
        Metric::Temperature => (-10.0, 40.0),
    }
}

pub fn colour_for(metric: Metric, value: f32) -> Rgba {
    let (min, max) = metric_domain(metric);
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);

    match metric {
        Metric::Cloud => lerp3(t, (0x1a, 0x3a, 0x5c), (0xbb, 0xdf, 0xeb), (0xe8, 0xee, 0xf4)),
        Metric::Sunlight => lerp2(t, (0x0d, 0x15, 0x20), (0xf5, 0xc8, 0x42)),
        Metric::Rain => lerp2(t, (0x1e, 0x28, 0x36), (0x2d, 0xd4, 0xbf)),
        Metric::Wind => lerp2(t, (0x3b, 0x2d, 0x5c), (0xf5, 0x9e, 0x42)),
        Metric::Temperature => diverging_temp(t),
    }
}

fn lerp2(t: f32, low: (u8, u8, u8), high: (u8, u8, u8)) -> Rgba {
    Rgba {
        r: lerp_u8(t, low.0, high.0),
        g: lerp_u8(t, low.1, high.1),
        b: lerp_u8(t, low.2, high.2),
        a: 255,
    }
}

fn lerp3(t: f32, a: (u8, u8, u8), b: (u8, u8, u8), c: (u8, u8, u8)) -> Rgba {
    if t < 0.5 {
        lerp2(t * 2.0, a, b)
    } else {
        lerp2((t - 0.5) * 2.0, b, c)
    }
}

fn diverging_temp(t: f32) -> Rgba {
    // cold blue → hot red (simplified RdYlBu)
    if t < 0.5 {
        lerp2(t * 2.0, (0x3b, 0x4c, 0xc0), (0xf7, 0xf7, 0xf7))
    } else {
        lerp2((t - 0.5) * 2.0, (0xf7, 0xf7, 0xf7), (0xb4, 0x04, 0x26))
    }
}

fn lerp_u8(t: f32, a: u8, b: u8) -> u8 {
    let af = f32::from(a);
    let bf = f32::from(b);
    u8_from_f32_rounded(af + (bf - af) * t)
}

pub fn lerp_rgb(t: f32, low: (u8, u8, u8), high: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        lerp_u8(t, low.0, high.0),
        lerp_u8(t, low.1, high.1),
        lerp_u8(t, low.2, high.2),
    )
}
