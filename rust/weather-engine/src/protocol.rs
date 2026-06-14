use crate::effect::EffectCommand;
use crate::event::AppEvent;
use crate::view_model::ViewModel;
use engine_kernel::ViewModelPatch;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "typegen",
    ts(tag = "kind", rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorkerInput {
    Init {
        #[serde(rename = "weatherBundle")]
        #[cfg_attr(feature = "typegen", ts(rename = "weatherBundle"))]
        #[serde(with = "serde_bytes")]
        #[cfg_attr(feature = "typegen", ts(type = "Uint8Array"))]
        weather_bundle: Vec<u8>,
    },
    Event { event: AppEvent },
}

/// Worker ↔ main metadata (patches, state effects). Presentation output uses a sidecar.
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "typegen",
    ts(tag = "kind", rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorkerOutput {
    Initialized {
        #[serde(rename = "viewModel")]
        #[cfg_attr(feature = "typegen", ts(rename = "viewModel"))]
        view_model: ViewModel,
        effects: Vec<EffectCommand>,
    },
    Response {
        patches: Vec<ViewModelPatch>,
        effects: Vec<EffectCommand>,
        diagnostics: Vec<String>,
    },
    Error { message: String },
}

pub fn encode_input(input: &WorkerInput) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(input, &mut buf).expect("encode WorkerInput");
    buf
}

pub fn decode_input(bytes: &[u8]) -> Result<WorkerInput, String> {
    ciborium::from_reader(bytes).map_err(|e| e.to_string())
}

pub fn encode_output(output: &WorkerOutput) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(output, &mut buf).expect("encode WorkerOutput");
    buf
}

pub fn decode_output(bytes: &[u8]) -> Result<WorkerOutput, String> {
    ciborium::from_reader(bytes).map_err(|e| e.to_string())
}
