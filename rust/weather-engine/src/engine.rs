use engine_kernel::diff_serializable_checked;

use crate::data::load_cities_from_bundle;
use crate::effect::EffectCommand;
use crate::geometry::build_geometry_wire;
use crate::protocol::{WorkerInput, WorkerOutput};
use crate::state::{AppState, FRAME_SIZE};
use crate::update::reduce;
use crate::view_model::{select_view_model, ViewModel};

pub struct Engine {
    state: AppState,
    view_model: ViewModel,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let state = AppState::initial(vec![]);
        let view_model = select_view_model(&state);
        Self { state, view_model }
    }

    pub fn dispatch(&mut self, input: &WorkerInput) -> WorkerOutput {
        match input {
            WorkerInput::Init { weather_bundle } => {
                let cities = match load_cities_from_bundle(weather_bundle) {
                    Ok(c) => c,
                    Err(message) => {
                        return WorkerOutput::Error { message };
                    }
                };
                self.state = AppState::initial(cities);
                self.view_model = select_view_model(&self.state);
                WorkerOutput::Initialized {
                    view_model: self.view_model.clone(),
                    effects: vec![],
                }
            }
            WorkerInput::Event { event } => {
                let prev_vm = self.view_model.clone();
                let transition = reduce(&mut self.state, event);
                self.view_model = select_view_model(&self.state);
                let patches = diff_serializable_checked(&prev_vm, &self.view_model);
                let mut effects = transition.effects;
                if transition.needs_frame {
                    if let Some(draw_wire) = self.build_geometry_wire() {
                        effects.push(EffectCommand::RenderSpiral {
                            width: FRAME_SIZE,
                            height: FRAME_SIZE,
                            draw_wire,
                        });
                    }
                }
                WorkerOutput::Response {
                    patches,
                    effects,
                    diagnostics: vec![],
                }
            }
        }
    }

    fn build_geometry_wire(&self) -> Option<Vec<u8>> {
        let city = self.state.selected_city()?;
        Some(build_geometry_wire(
            &city.metrics,
            self.state.selected_metric,
            self.state.selected_view_mode,
            city.start_unix,
            city.hour_step,
            self.state.selected_zoom,
            FRAME_SIZE,
            0,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::EffectCommand;
    use crate::event::AppEvent;
    use crate::geometry::wire_byte_len;

    use crate::data::weather_bundle_path;

    fn init_input() -> WorkerInput {
        let bundle = std::fs::read(weather_bundle_path()).expect("weather bundle");
        WorkerInput::Init {
            weather_bundle: bundle,
        }
    }

    #[test]
    fn init_is_fast_no_render_effect() {
        let mut engine = Engine::new();
        let out = engine.dispatch(&init_input());
        match out {
            WorkerOutput::Initialized { effects, .. } => assert!(effects.is_empty()),
            other => panic!("expected initialized, got {other:?}"),
        }
    }

    #[test]
    fn request_frame_plans_render_spiral_effect() {
        let mut engine = Engine::new();
        let _ = engine.dispatch(&init_input());
        let out = engine.dispatch(&WorkerInput::Event {
            event: AppEvent::RequestFrame,
        });
        match out {
            WorkerOutput::Response { effects, .. } => {
                let render = effects
                    .iter()
                    .find(|effect| matches!(effect, EffectCommand::RenderSpiral { .. }))
                    .expect("renderSpiral effect");
                let EffectCommand::RenderSpiral { draw_wire, width, height } = render else {
                    panic!("expected renderSpiral");
                };
                assert_eq!(*width, FRAME_SIZE);
                assert_eq!(*height, FRAME_SIZE);
                let count = u32::from_le_bytes(draw_wire[0..4].try_into().unwrap());
                assert_eq!(draw_wire.len(), wire_byte_len(count));
                assert_eq!(count, 8784);
            }
            other => panic!("expected response, got {other:?}"),
        }
    }
}
