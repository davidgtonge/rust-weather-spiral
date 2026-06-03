use serde::{Deserialize, Serialize};

use crate::state::{Metric, Zoom};
use crate::view_mode::ViewMode;

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(tag = "type", rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppEvent {
    RequestFrame,
    CitySelected {
        #[serde(rename = "cityId")]
        #[cfg_attr(feature = "typegen", ts(rename = "cityId"))]
        city_id: String,
    },
    MetricSelected { metric: Metric },
    ZoomSelected { zoom: Zoom },
    ViewModeSelected {
        #[serde(rename = "viewMode")]
        #[cfg_attr(feature = "typegen", ts(rename = "viewMode"))]
        view_mode: ViewMode,
    },
    Tick,
}
