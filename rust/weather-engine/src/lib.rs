mod colour;
mod daylight;
mod data;
mod draw;
mod effect;
mod engine;
mod event;
mod frame;
mod geometry;
mod protocol;
mod spiral;
mod state;
mod update;
mod view_mode;
mod view_model;
mod views;

#[cfg(feature = "typegen")]
mod typegen;

#[cfg(feature = "typegen")]
pub use typegen::export_types;

pub use effect::EffectCommand;
pub use engine::Engine;
pub use event::AppEvent;
pub use engine_kernel::ViewModelPatch;
pub use frame::SpiralFrame;
pub use protocol::{decode_input, decode_output, encode_input, encode_output, WorkerInput, WorkerOutput};
pub use view_model::ViewModel;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WeatherEngine {
    inner: Engine,
}

impl Default for WeatherEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WeatherEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Engine::new(),
        }
    }

    pub fn handle_input(&mut self, payload: &[u8]) -> Vec<u8> {
        match decode_input(payload) {
            Ok(input) => encode_output(&self.inner.dispatch(&input)),
            Err(message) => encode_output(&WorkerOutput::Error { message }),
        }
    }
}
