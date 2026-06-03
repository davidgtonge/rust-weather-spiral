use crate::effect::EffectCommand;
use crate::event::AppEvent;
use crate::state::AppState;

pub struct Transition {
    pub effects: Vec<EffectCommand>,
    pub needs_frame: bool,
}

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Transition {
    match event {
        AppEvent::RequestFrame => Transition {
            effects: vec![],
            needs_frame: true,
        },
        AppEvent::CitySelected { city_id } => {
            if state.cities.iter().any(|c| c.id == *city_id) {
                state.selected_city_id.clone_from(city_id);
            }
            Transition {
                effects: vec![],
                needs_frame: true,
            }
        }
        AppEvent::MetricSelected { metric } => {
            state.selected_metric = *metric;
            Transition {
                effects: vec![],
                needs_frame: true,
            }
        }
        AppEvent::ZoomSelected { zoom } => {
            state.selected_zoom = *zoom;
            Transition {
                effects: vec![],
                needs_frame: true,
            }
        }
        AppEvent::ViewModeSelected { view_mode } => {
            state.selected_view_mode = *view_mode;
            Transition {
                effects: vec![],
                needs_frame: true,
            }
        }
        AppEvent::Tick => Transition {
            effects: vec![],
            needs_frame: false,
        },
    }
}
