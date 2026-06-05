import init, { WeatherEngine } from "../../pkg/weather_engine.js";
import { drainPresentationEffects } from "./drain-presentation-effects";
import type { PresentationSidecar } from "./drain-presentation-effects";
import type { WorkerTimings } from "./timings";

type InboundMessage = { bytes: ArrayBuffer };
type OutboundMessage = {
  bytes: ArrayBuffer;
  presentation?: PresentationSidecar;
  timings: WorkerTimings;
};

const engineReady = init().then(() => new WeatherEngine());

self.onmessage = async (event: MessageEvent<InboundMessage>) => {
  const workerStart = performance.now();
  const engine = await engineReady;
  const inbound = new Uint8Array(event.data.bytes);

  const wasmStart = performance.now();
  const meta = engine.handle_input(inbound);
  const wasmMs = performance.now() - wasmStart;

  const metaBuffer = meta.buffer.slice(
    meta.byteOffset,
    meta.byteOffset + meta.byteLength,
  ) as ArrayBuffer;

  const drained = drainPresentationEffects(metaBuffer);

  const transfers: Transferable[] = [drained.bytes];
  const out: OutboundMessage = {
    bytes: drained.bytes,
    timings: {
      wasmMs,
      geometryKb: drained.geometryKb,
      canvasMs: drained.canvasMs,
      workerTotalMs: 0,
    },
  };

  if (drained.presentation) {
    out.presentation = drained.presentation;
    out.timings.canvasProfile = drained.canvasProfile;
    transfers.push(drained.presentation.bitmap);
  }

  out.timings.workerTotalMs = performance.now() - workerStart;

  self.postMessage(out, { transfer: transfers });
};
