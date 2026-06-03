use serde::{Deserialize, Serialize};

use crate::colour::metric_domain;
use crate::state::{AppState, Metric, Zoom, FRAME_SIZE};
use crate::view_mode::ViewMode;

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CityOption {
    pub id: String,
    pub label: String,
}

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewModel {
    pub city_label: String,
    pub metric_label: String,
    pub zoom_label: String,
    pub view_mode_label: String,
    pub view_mode_description: String,
    pub cities: Vec<CityOption>,
    pub selected_city_id: String,
    pub selected_metric: Metric,
    pub selected_zoom: Zoom,
    pub selected_view_mode: ViewMode,
    pub frame_width: u32,
    pub frame_height: u32,
    pub color_domain_min: f32,
    pub color_domain_max: f32,
    pub color_unit: String,
    pub loading: bool,
    pub show_metric_tabs: bool,
}

pub fn select_view_model(state: &AppState) -> ViewModel {
    let loading = state.cities.is_empty();
    let city_label = state
        .selected_city()
        .map(|c| c.label.clone())
        .unwrap_or_default();
    let (color_domain_min, color_domain_max) = metric_domain(state.selected_metric);
    let color_unit = match state.selected_metric {
        Metric::Cloud => "%".to_string(),
        Metric::Sunlight => "W/m²".to_string(),
        Metric::Rain => "mm".to_string(),
        Metric::Wind => "m/s".to_string(),
        Metric::Temperature => "°C".to_string(),
    };
    let mode = state.selected_view_mode;
    let show_metric_tabs = mode == ViewMode::Metric;

    ViewModel {
        city_label,
        metric_label: state.selected_metric.label().to_string(),
        zoom_label: state.selected_zoom.label().to_string(),
        view_mode_label: mode.label().to_string(),
        view_mode_description: mode.description().to_string(),
        cities: state
            .cities
            .iter()
            .map(|c| CityOption {
                id: c.id.clone(),
                label: c.label.clone(),
            })
            .collect(),
        selected_city_id: state.selected_city_id.clone(),
        selected_metric: state.selected_metric,
        selected_zoom: state.selected_zoom,
        selected_view_mode: mode,
        frame_width: FRAME_SIZE,
        frame_height: FRAME_SIZE,
        color_domain_min,
        color_domain_max,
        color_unit,
        loading,
        show_metric_tabs,
    }
}
