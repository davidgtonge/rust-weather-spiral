use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(tag = "type", rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EffectCommand {
    /// Terminal presentation effect — worker shell rasterises; never returns to the engine.
    RenderSpiral {
        width: u32,
        height: u32,
        #[serde(rename = "drawWire")]
        #[serde(with = "serde_bytes")]
        #[cfg_attr(feature = "typegen", ts(type = "Uint8Array"))]
        draw_wire: Vec<u8>,
    },
    TimerStart {
        id: String,
        #[serde(rename = "intervalMs")]
        interval_ms: u32,
    },
    TimerStop { id: String },
}
