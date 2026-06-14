# rust-weather-spiral

Weather spiral demo — Rust/Wasm engine, Open-Meteo offline data, canvas blit.

> **Weather Spiral: visualising seasonal weather with Rust, Wasm, CBOR, and canvas**

## Quick start

```bash
git clone --recurse-submodules https://github.com/davidgtonge/rust-weather-spiral.git
cd rust-weather-spiral
npm install
npm run dev      # http://localhost:5173 — Vite + Wasm worker
npm run build    # production bundle
npm run test:rust
```

Weather series ship as **`weather.bundle.cbor`** (~0.7 MB dense f32 blobs) embedded in Wasm. The worker draws 8,784 segments at 1024² via OffscreenCanvas 2D (~15–20 ms).

## Architecture

```txt
Preact UI → CBOR worker → Rust/Wasm (weather-engine)
  → view-model patches + compact geometry wire
  → worker OffscreenCanvas 2D → ImageBitmap blit
```

Built on [@dtonge/engine-shell](https://github.com/davidgtonge/engine-shell) — shared Rust/Wasm + TypeScript worker scaffold (CBOR wire, view-model patches).

## Data

The repo includes a pre-built **`data/weather.bundle.cbor`**. Hourly JSON sources are not committed (regenerate locally):

```bash
npm run fetch:weather       # download Open-Meteo JSON + rebuild CBOR bundle
npm run build:weather-cbor  # JSON → weather.bundle.cbor (Wasm embed)
npm run validate:data       # smoke test JSON (after fetch)
npm run validate:cbor       # smoke test bundle
```

Cities: Bristol, Ljubljana, Nice, Reykjavik. The CBOR bundle is `include_bytes!` in the Wasm binary (one decode; f32 arrays as dense byte blobs).

## Engine modules

```txt
rust/weather-engine/src/
  data.rs      # load embedded CBOR bundle
  spiral.rs    # layout (port of react-spiral)
  colour.rs    # per-metric palettes
  geometry.rs  # segment wire for Canvas2D
  engine.rs    # dispatch + geometry on visual change
```

## Status

| Phase | Status |
|-------|--------|
| 1 Data | Complete |
| 2 Engine + rendering | Complete |
| 3 Interaction | Not started (playback scrubber) |
| 4 Polish | Not started |

## Attribution

Weather data by [Open-Meteo](https://open-meteo.com/). Historical archive API.

## License

MIT
