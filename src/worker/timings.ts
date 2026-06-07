/** Per-stage breakdown from `drawGeometryToBitmap` (dev only). */
export type CanvasDrawProfile = {
  parseMs: number;
  clearMs: number;
  loopMs: number;
  /** Extrapolated from post-draw microbench (1.5k iters × segment count). */
  transformMs: number;
  styleMs: number;
  fillMs: number;
  bitmapMs: number;
  segmentCount: number;
  fillStyleChanges: number;
  alphaChanges: number;
};

export type WorkerTimings = {
  wasmMs: number;
  geometryKb: number;
  canvasMs: number;
  workerTotalMs: number;
  canvasProfile?: CanvasDrawProfile;
};

export type BlitReport = {
  ms: number;
  frames: number;
};

export type FrameTimings = WorkerTimings & {
  decodeMs: number;
  roundTripMs: number;
  blitMs?: number;
  blitFrames?: number;
};

export function formatMs(ms: number): string {
  return `${ms.toFixed(2)} ms`;
}

export function logTimings(label: string, timings: Partial<FrameTimings>): void {
  if (!import.meta.env.DEV) return;

  const rows: Record<string, string> = {};
  if (timings.roundTripMs != null) rows["round-trip (main)"] = formatMs(timings.roundTripMs);
  if (timings.decodeMs != null) rows["decode meta (main)"] = formatMs(timings.decodeMs);
  if (timings.blitMs != null) rows["blit canvas (main)"] = formatMs(timings.blitMs);
  if (timings.blitFrames != null) rows["blit frames (main)"] = String(timings.blitFrames);
  if (timings.wasmMs != null) rows["wasm handle_input"] = formatMs(timings.wasmMs);
  if (timings.geometryKb != null) rows["geometry wire"] = `${timings.geometryKb.toFixed(1)} KB`;
  if (timings.canvasMs != null) rows["canvas2d draw (worker)"] = formatMs(timings.canvasMs);
  if (timings.workerTotalMs != null) rows["worker total"] = formatMs(timings.workerTotalMs);

  const p = timings.canvasProfile;
  if (p) {
    rows["  ↳ parse wire"] = formatMs(p.parseMs);
    rows["  ↳ clear bg"] = formatMs(p.clearMs);
    rows["  ↳ segment loop"] = formatMs(p.loopMs);
    rows["      ↳ setTransform (bench)"] = formatMs(p.transformMs);
    rows["      ↳ style/alpha (bench)"] = formatMs(p.styleMs);
    rows["      ↳ fillRect (bench)"] = formatMs(p.fillMs);
    rows["  ↳ transferToImageBitmap"] = formatMs(p.bitmapMs);
    rows["  ↳ segments"] = String(p.segmentCount);
    rows["  ↳ fillStyle changes"] = String(p.fillStyleChanges);
    rows["  ↳ alpha changes"] = String(p.alphaChanges);
  }

  console.groupCollapsed(`[weather-spiral] ${label}`);
  console.table(rows);
  console.groupEnd();
}
