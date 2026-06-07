/**
 * Canvas2D draw-command executor and bitmap pipeline — geometry from Rust, Canvas2D in TS.
 */

import { parseGeometryWire } from "./geometry-wire";
import {
  FL_CANVAS_SPACE,
  OP_BEGIN_CLIP,
  OP_END_CLIP,
  OP_FILL_CIRCLE,
  OP_FILL_ELLIPSE,
  OP_FILL_RECT,
  OP_STROKE_ARC,
  OP_STROKE_LINE,
  OP_STROKE_RECT,
  type DrawWire,
} from "./geometry-wire";
import type { CanvasDrawProfile } from "./timings";

type Ctx = OffscreenCanvasRenderingContext2D;
type ColorCache = Map<number, string>;

const BG = "#0a0c10";
const BENCH_ITERS = 1500;

export type SpiralBitmap = {
  bitmap: ImageBitmap;
  width: number;
  height: number;
  profile?: CanvasDrawProfile;
};

type Scratch = {
  canvas: OffscreenCanvas | null;
  ctx: OffscreenCanvasRenderingContext2D | null;
  colorCache: Map<number, string>;
};

const scratch: Scratch = { canvas: null, ctx: null, colorCache: new Map() };

function fillStyleForRgb(
  rgb: number,
  ri: number,
  gi: number,
  bi: number,
  cache: ColorCache,
): string {
  const cached = cache.get(rgb);
  if (cached !== undefined) return cached;
  const color = `rgb(${ri},${gi},${bi})`;
  cache.set(rgb, color);
  return color;
}

function rgbaStyle(
  r: number,
  g: number,
  b: number,
  a: number,
  cache: ColorCache,
): string {
  if (a === 255) {
    return fillStyleForRgb((r << 16) | (g << 8) | b, r, g, b, cache);
  }
  return `rgba(${r},${g},${b},${a / 255})`;
}

function applyTransform(ctx: Ctx, cmd: DrawWire["commands"][number]): void {
  if (cmd.flags & FL_CANVAS_SPACE) {
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    return;
  }
  ctx.setTransform(cmd.cos, cmd.sin, -cmd.sin, cmd.cos, cmd.tx, cmd.ty);
}

function executeCommand(ctx: Ctx, cmd: DrawWire["commands"][number], cache: ColorCache): void {
  switch (cmd.op) {
    case OP_BEGIN_CLIP:
      ctx.save();
      ctx.beginPath();
      ctx.rect(cmd.p0, cmd.p1, cmd.p2, cmd.p3);
      ctx.clip();
      return;
    case OP_END_CLIP:
      ctx.restore();
      return;
    default:
      break;
  }

  applyTransform(ctx, cmd);
  ctx.globalAlpha = cmd.a === 255 ? 1 : cmd.a / 255;

  switch (cmd.op) {
    case OP_FILL_RECT:
      ctx.fillStyle = rgbaStyle(cmd.r, cmd.g, cmd.b, cmd.a, cache);
      ctx.fillRect(cmd.p0, cmd.p1, cmd.p2, cmd.p3);
      break;
    case OP_STROKE_LINE:
      ctx.strokeStyle = rgbaStyle(cmd.r, cmd.g, cmd.b, cmd.a, cache);
      ctx.lineWidth = cmd.lineWidth;
      ctx.lineCap = "round";
      ctx.beginPath();
      ctx.moveTo(cmd.p0, cmd.p1);
      ctx.lineTo(cmd.p2, cmd.p3);
      ctx.stroke();
      break;
    case OP_STROKE_ARC:
      ctx.strokeStyle = rgbaStyle(cmd.r, cmd.g, cmd.b, cmd.a, cache);
      ctx.lineWidth = cmd.lineWidth;
      ctx.beginPath();
      ctx.arc(cmd.p0, cmd.p1, cmd.p2, cmd.p3, cmd.p4);
      ctx.stroke();
      break;
    case OP_FILL_CIRCLE:
      ctx.fillStyle = rgbaStyle(cmd.r, cmd.g, cmd.b, cmd.a, cache);
      ctx.beginPath();
      ctx.arc(cmd.p0, cmd.p1, cmd.p2, 0, Math.PI * 2);
      ctx.fill();
      break;
    case OP_FILL_ELLIPSE:
      ctx.fillStyle = rgbaStyle(cmd.r, cmd.g, cmd.b, cmd.a, cache);
      ctx.beginPath();
      ctx.ellipse(cmd.p0, cmd.p1, cmd.p2, cmd.p3, 0, 0, Math.PI * 2);
      ctx.fill();
      break;
    case OP_STROKE_RECT:
      ctx.strokeStyle = rgbaStyle(cmd.r, cmd.g, cmd.b, cmd.a, cache);
      ctx.lineWidth = cmd.lineWidth;
      ctx.strokeRect(cmd.p0, cmd.p1, cmd.p2, cmd.p3);
      break;
    default:
      break;
  }
}

/** Execute pre-built draw commands from Rust (no view-mode logic in TS). */
export function drawGeometryByMode(
  ctx: Ctx,
  wire: DrawWire,
  cache: ColorCache = new Map(),
  _canvasW = 1024,
  _canvasH = 1024,
): void {
  for (let i = 0; i < wire.count; i++) {
    executeCommand(ctx, wire.commands[i]!, cache);
  }
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.globalAlpha = 1;
}

function acquireCanvas(
  width: number,
  height: number,
): { canvas: OffscreenCanvas; ctx: OffscreenCanvasRenderingContext2D } {
  if (!scratch.canvas || scratch.canvas.width !== width || scratch.canvas.height !== height) {
    scratch.canvas = new OffscreenCanvas(width, height);
    scratch.ctx = scratch.canvas.getContext("2d", {
      alpha: false,
      desynchronized: true,
    });
  }
  if (!scratch.ctx) {
    throw new Error("OffscreenCanvas 2d context unavailable");
  }
  return { canvas: scratch.canvas, ctx: scratch.ctx };
}

function benchmarkLoopPhases(
  ctx: OffscreenCanvasRenderingContext2D,
  count: number,
): Pick<CanvasDrawProfile, "transformMs" | "styleMs" | "fillMs"> {
  let t0 = performance.now();
  for (let k = 0; k < BENCH_ITERS; k++) {
    ctx.setTransform(1, 0, 0, 1, 100, 100);
  }
  const transformMs = ((performance.now() - t0) / BENCH_ITERS) * count;

  ctx.setTransform(1, 0, 0, 1, 0, 0);
  t0 = performance.now();
  for (let k = 0; k < BENCH_ITERS; k++) {
    ctx.globalAlpha = 1;
    ctx.fillStyle = "rgb(128,128,128)";
  }
  const styleMs = ((performance.now() - t0) / BENCH_ITERS) * count;

  ctx.setTransform(1, 0, 0, 1, 100, 100);
  ctx.globalAlpha = 1;
  ctx.fillStyle = "rgb(128,128,128)";
  t0 = performance.now();
  for (let k = 0; k < BENCH_ITERS; k++) {
    ctx.fillRect(0, 0, 10, 2);
  }
  const fillMs = ((performance.now() - t0) / BENCH_ITERS) * count;

  return { transformMs, styleMs, fillMs };
}

export function drawGeometryToBitmap(
  geometry: ArrayBuffer,
  width: number,
  height: number,
): SpiralBitmap {
  const profileDraw = import.meta.env.DEV;
  const { canvas, ctx } = acquireCanvas(width, height);

  const parseStart = profileDraw ? performance.now() : 0;
  const wire = parseGeometryWire(geometry);
  const parseMs = profileDraw ? performance.now() - parseStart : 0;

  if (typeof ctx.reset === "function") {
    ctx.reset();
  }

  const clearStart = profileDraw ? performance.now() : 0;
  ctx.fillStyle = BG;
  ctx.fillRect(0, 0, width, height);
  const clearMs = profileDraw ? performance.now() - clearStart : 0;

  const loopStart = profileDraw ? performance.now() : 0;
  drawGeometryByMode(ctx, wire, scratch.colorCache, width, height);
  const loopMs = profileDraw ? performance.now() - loopStart : 0;

  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.globalAlpha = 1;

  const bitmapStart = profileDraw ? performance.now() : 0;
  const bitmap = canvas.transferToImageBitmap();
  const bitmapMs = profileDraw ? performance.now() - bitmapStart : 0;

  const result: SpiralBitmap = { bitmap, width, height };
  if (profileDraw) {
    const phases = benchmarkLoopPhases(ctx, wire.count);
    result.profile = {
      parseMs,
      clearMs,
      loopMs,
      ...phases,
      bitmapMs,
      segmentCount: wire.count,
      fillStyleChanges: 0,
      alphaChanges: 0,
    };
  }

  return result;
}
