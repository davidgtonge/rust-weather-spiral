/**
 * Worker-shell presentation effects — terminal outputs that never return to Wasm.
 */

import { Decoder, encode } from "cbor-x";
import type { EffectCommand, WorkerOutput } from "../protocol/types";
import { drawGeometryToBitmap } from "./spiral-canvas";
import type { CanvasDrawProfile } from "./timings";

const decoder = new Decoder({ copyBuffers: false, useRecords: false });

export type PresentationSidecar = {
  kind: "bitmap";
  bitmap: ImageBitmap;
  width: number;
  height: number;
};

export type DrainPresentationResult = {
  bytes: ArrayBuffer;
  presentation?: PresentationSidecar;
  geometryKb: number;
  canvasMs: number;
  canvasProfile?: CanvasDrawProfile;
};

function isRenderSpiral(
  effect: EffectCommand,
): effect is Extract<EffectCommand, { type: "renderSpiral" }> {
  return effect.type === "renderSpiral";
}

function toArrayBuffer(drawWire: Uint8Array): ArrayBuffer {
  if (drawWire.byteOffset === 0 && drawWire.byteLength === drawWire.buffer.byteLength) {
    return drawWire.buffer as ArrayBuffer;
  }
  return drawWire.buffer.slice(
    drawWire.byteOffset,
    drawWire.byteOffset + drawWire.byteLength,
  ) as ArrayBuffer;
}

function stripRenderSpiralEffects(effects: EffectCommand[]): {
  remaining: EffectCommand[];
  render?: Extract<EffectCommand, { type: "renderSpiral" }>;
} {
  const remaining: EffectCommand[] = [];
  let render: Extract<EffectCommand, { type: "renderSpiral" }> | undefined;

  for (const effect of effects) {
    if (isRenderSpiral(effect)) {
      render = effect;
      continue;
    }
    remaining.push(effect);
  }

  return { remaining, render };
}

function encodeWorkerOutput(output: WorkerOutput): ArrayBuffer {
  const encoded = encode(output) as Uint8Array;
  return encoded.buffer.slice(
    encoded.byteOffset,
    encoded.byteOffset + encoded.byteLength,
  ) as ArrayBuffer;
}

/** Decode Wasm CBOR, execute terminal render effects, re-encode without them. */
export function drainPresentationEffects(cborBytes: ArrayBuffer): DrainPresentationResult {
  const output = decoder.decode(new Uint8Array(cborBytes)) as WorkerOutput;

  let presentation: PresentationSidecar | undefined;
  let geometryKb = 0;
  let canvasMs = 0;
  let canvasProfile: CanvasDrawProfile | undefined;
  let stripped: WorkerOutput = output;

  if (output.kind === "initialized") {
    const { remaining, render } = stripRenderSpiralEffects(output.effects);
    if (render) {
      const geomBuffer = toArrayBuffer(render.drawWire);
      geometryKb = geomBuffer.byteLength / 1024;
      const canvasStart = performance.now();
      const drawn = drawGeometryToBitmap(geomBuffer, render.width, render.height);
      canvasMs = performance.now() - canvasStart;
      canvasProfile = drawn.profile;
      presentation = {
        kind: "bitmap",
        bitmap: drawn.bitmap,
        width: drawn.width,
        height: drawn.height,
      };
    }
    stripped = { ...output, effects: remaining };
  } else if (output.kind === "response") {
    const { remaining, render } = stripRenderSpiralEffects(output.effects);
    if (render) {
      const geomBuffer = toArrayBuffer(render.drawWire);
      geometryKb = geomBuffer.byteLength / 1024;
      const canvasStart = performance.now();
      const drawn = drawGeometryToBitmap(geomBuffer, render.width, render.height);
      canvasMs = performance.now() - canvasStart;
      canvasProfile = drawn.profile;
      presentation = {
        kind: "bitmap",
        bitmap: drawn.bitmap,
        width: drawn.width,
        height: drawn.height,
      };
    }
    stripped = { ...output, effects: remaining };
  }

  return {
    bytes: encodeWorkerOutput(stripped),
    presentation,
    geometryKb,
    canvasMs,
    canvasProfile,
  };
}
