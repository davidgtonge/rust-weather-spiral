use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(rename_all = "camelCase"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewMode {
    /// Single-metric coloured segments (original view).
    Metric,
    /// Sky colour + rain beads + wind whiskers (multi-layer).
    Tapestry,
    /// Four parallel tracks: sun, cloud, rain, wind.
    Ribbon,
    /// Dominant-condition palette with overlay emphasis.
    Condition,
    /// Custom weather glyph per segment.
    Glyphs,
    /// Twelve monthly petals — seasonal mandala.
    Mandala,
    /// Angle = day of year, radius = hour of day.
    Fingerprint,
    /// Lit hours only — seasonal sunprint ring (dawn inside, dusk outside).
    Daylight,
}

impl ViewMode {
    pub const ALL: [ViewMode; 8] = [
        ViewMode::Metric,
        ViewMode::Tapestry,
        ViewMode::Ribbon,
        ViewMode::Condition,
        ViewMode::Glyphs,
        ViewMode::Mandala,
        ViewMode::Fingerprint,
        ViewMode::Daylight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Metric => "Metric",
            ViewMode::Tapestry => "Tapestry",
            ViewMode::Ribbon => "Ribbon",
            ViewMode::Condition => "Condition",
            ViewMode::Glyphs => "Split Glyphs",
            ViewMode::Mandala => "Mandala",
            ViewMode::Fingerprint => "Fingerprint",
            ViewMode::Daylight => "Daylight",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            ViewMode::Metric => "One metric mapped to segment colour",
            ViewMode::Tapestry => "Sky, rain beads, and wind whiskers",
            ViewMode::Ribbon => "Four vinyl grooves for sun, cloud, rain, wind",
            ViewMode::Condition => "Dominant weather type at a glance",
            ViewMode::Glyphs => "Sky and rain above, temperature and wind below",
            ViewMode::Mandala => "Twelve petals — one per month",
            ViewMode::Fingerprint => "Day around the circle, hour by radius",
            ViewMode::Daylight => "Only sunlit hours — the ring widens and narrows with the seasons",
        }
    }

    pub const fn as_u32(self) -> u32 {
        match self {
            ViewMode::Metric => 0,
            ViewMode::Tapestry => 1,
            ViewMode::Ribbon => 2,
            ViewMode::Condition => 3,
            ViewMode::Glyphs => 4,
            ViewMode::Mandala => 5,
            ViewMode::Fingerprint => 6,
            ViewMode::Daylight => 7,
        }
    }

    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => ViewMode::Tapestry,
            2 => ViewMode::Ribbon,
            3 => ViewMode::Condition,
            4 => ViewMode::Glyphs,
            5 => ViewMode::Mandala,
            6 => ViewMode::Fingerprint,
            7 => ViewMode::Daylight,
            _ => ViewMode::Metric,
        }
    }
}
