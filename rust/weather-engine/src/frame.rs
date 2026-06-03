use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(rename_all = "camelCase"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpiralFrame {
    pub width: u32,
    pub height: u32,
    #[serde(with = "serde_bytes")]
    #[cfg_attr(feature = "typegen", ts(type = "Uint8Array"))]
    pub pixels: Vec<u8>,
}
