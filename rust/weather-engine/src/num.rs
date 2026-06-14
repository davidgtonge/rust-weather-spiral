//! Canvas and colour conversions — centralised casts for layout math.

#[inline]
#[must_use]
pub fn f32_from_u32(v: u32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    {
        v as f32
    }
}

#[inline]
#[must_use]
pub fn f32_from_usize(v: usize) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    {
        v as f32
    }
}

#[inline]
#[must_use]
pub fn u8_from_f32_rounded(v: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        v.round() as u8
    }
}

#[inline]
#[must_use]
pub fn u8_from_alpha(v: f32) -> u8 {
    u8_from_f32_rounded(v.clamp(0.0, 1.0) * 255.0)
}

#[inline]
#[must_use]
pub fn u32_from_usize(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

#[inline]
#[must_use]
pub fn u32_from_f32_rounded(v: f32) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        v.round().max(0.0) as u32
    }
}
