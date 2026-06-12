/**
 * Worker wire + canvas contracts — must stay in sync with rust/weather-engine/src/draw.rs
 */
import { Decoder, encode } from "cbor-x";
import { describe, expect, it, vi } from "vitest";
import type { EffectCommand, WorkerOutput } from "../../protocol/types";
import {
  BYTES_PER_COMMAND,
  BYTES_PER_SEGMENT,
  FL_CANVAS_SPACE,
  GEOMETRY_HEADER,
  OP_FILL_CIRCLE,
  OP_FILL_RECT,
  parseGeometryWire,
  type DrawWire,
  wireByteLength,
} from "../geometry-wire";
import { drawGeometryByMode } from "../spiral-canvas";

vi.mock("../spiral-canvas", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../spiral-canvas")>();
  return {
    ...actual,
    drawGeometryToBitmap: vi.fn(() => ({
      bitmap: { close: () => {} } as ImageBitmap,
      width: 64,
      height: 64,
    })),
  };
});

import { drainPresentationEffects } from "../drain-presentation-effects";

const decoder = new Decoder({ copyBuffers: false, useRecords: false });

function goldenDrawFixture(): ArrayBuffer {
  const count = 2;
  const buf = new ArrayBuffer(wireByteLength(count));
  const dv = new DataView(buf);

  dv.setUint32(0, count, true);
  dv.setFloat32(4, 750, true);
  dv.setFloat32(8, 750, true);
  dv.setUint32(12, 1, true);
  dv.setUint32(16, 1, true);

  const writeCmd = (
    index: number,
    op: number,
    flags: number,
    r: number,
    g: number,
    b: number,
    a: number,
    lineWidth: number,
    cos: number,
    sin: number,
    tx: number,
    ty: number,
    p0: number,
    p1: number,
    p2: number,
    p3: number,
    p4 = 0,
  ) => {
    const base = GEOMETRY_HEADER + index * BYTES_PER_COMMAND;
    dv.setUint8(base, op);
    dv.setUint8(base + 1, flags);
    dv.setUint8(base + 2, r);
    dv.setUint8(base + 3, g);
    dv.setUint8(base + 4, b);
    dv.setUint8(base + 5, a);
    dv.setFloat32(base + 8, lineWidth, true);
    dv.setFloat32(base + 12, cos, true);
    dv.setFloat32(base + 16, sin, true);
    dv.setFloat32(base + 20, tx, true);
    dv.setFloat32(base + 24, ty, true);
    dv.setFloat32(base + 28, p0, true);
    dv.setFloat32(base + 32, p1, true);
    dv.setFloat32(base + 36, p2, true);
    dv.setFloat32(base + 40, p3, true);
    dv.setFloat32(base + 44, p4, true);
  };

  writeCmd(0, OP_FILL_RECT, 0, 255, 0, 128, 255, 0, 1, 0, 100, 300, 0, 0, 10, 2);
  writeCmd(1, OP_FILL_CIRCLE, FL_CANVAS_SPACE, 245, 200, 66, 56, 0, 1, 0, 0, 0, 200, 300, 8, 0);

  return buf;
}

function brightGlowWire(): DrawWire {
  const buf = new ArrayBuffer(wireByteLength(1));
  const dv = new DataView(buf);
  dv.setUint32(0, 1, true);
  dv.setFloat32(4, 1024, true);
  dv.setFloat32(8, 1024, true);
  dv.setUint32(16, 5, true);

  const base = GEOMETRY_HEADER;
  dv.setUint8(base, OP_FILL_CIRCLE);
  dv.setUint8(base + 1, FL_CANVAS_SPACE);
  dv.setUint8(base + 2, 245);
  dv.setUint8(base + 3, 200);
  dv.setUint8(base + 4, 66);
  dv.setUint8(base + 5, 56);
  dv.setFloat32(base + 28, 200, true);
  dv.setFloat32(base + 32, 300, true);
  dv.setFloat32(base + 36, 8, true);

  return {
    count: 1,
    canvasW: 1024,
    canvasH: 1024,
    playheadIndex: 0,
    viewMode: "mandala",
    commands: [
      {
        op: OP_FILL_CIRCLE,
        flags: FL_CANVAS_SPACE,
        r: 245,
        g: 200,
        b: 66,
        a: 56,
        lineWidth: 0,
        cos: 1,
        sin: 0,
        tx: 0,
        ty: 0,
        p0: 200,
        p1: 300,
        p2: 8,
        p3: 0,
        p4: 0,
      },
    ],
  };
}

function encodeOutput(output: WorkerOutput): ArrayBuffer {
  const bytes = encode(output) as Uint8Array;
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}

describe("draw command wire layout", () => {
  it("wireByteLength is header + count × 48", () => {
    expect(wireByteLength(0)).toBe(20);
    expect(wireByteLength(2)).toBe(20 + 96);
  });

  it("parseGeometryWire reads header from golden fixture", () => {
    const wire = parseGeometryWire(goldenDrawFixture());
    expect(wire.count).toBe(2);
    expect(wire.canvasW).toBeCloseTo(750);
    expect(wire.canvasH).toBeCloseTo(750);
    expect(wire.playheadIndex).toBe(1);
    expect(wire.viewMode).toBe("tapestry");
  });

  it("parseGeometryWire reads command fields", () => {
    const wire = parseGeometryWire(goldenDrawFixture());
    expect(wire.commands[0]!.op).toBe(OP_FILL_RECT);
    expect(wire.commands[0]!.tx).toBeCloseTo(100);
    expect(wire.commands[0]!.p2).toBeCloseTo(10);
    expect(wire.commands[1]!.flags).toBe(FL_CANVAS_SPACE);
    expect(wire.commands[1]!.p0).toBeCloseTo(200);
    expect(wire.commands[1]!.p1).toBeCloseTo(300);
  });

  it("BYTES_PER_SEGMENT aliases command stride", () => {
    expect(BYTES_PER_SEGMENT).toBe(BYTES_PER_COMMAND);
    expect(BYTES_PER_COMMAND).toBe(48);
  });
});

describe("draw command executor", () => {
  it("draws canvas-space circle at anchor coordinates", () => {
    const wire = brightGlowWire();
    const arcs: Array<{ x: number; y: number; r: number }> = [];

    type FakeCtx = {
      fillStyle: string;
      globalAlpha: number;
      setTransform: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
      beginPath: () => void;
      arc: (x: number, y: number, r: number) => void;
      fill: () => void;
      fillRect: () => void;
      stroke: () => void;
      moveTo: () => void;
      lineTo: () => void;
      ellipse: () => void;
      strokeRect: () => void;
      save: () => void;
      restore: () => void;
      rect: () => void;
      clip: () => void;
    };

    const ctx: FakeCtx = {
      fillStyle: "",
      globalAlpha: 1,
      setTransform() {},
      beginPath() {},
      arc(x: number, y: number, r: number) {
        arcs.push({ x, y, r });
      },
      fill() {},
      fillRect() {},
      stroke() {},
      moveTo() {},
      lineTo() {},
      ellipse() {},
      strokeRect() {},
      save() {},
      restore() {},
      rect() {},
      clip() {},
    };

    drawGeometryByMode(ctx as unknown as OffscreenCanvasRenderingContext2D, wire);

    expect(arcs).toHaveLength(1);
    expect(arcs[0]!.x).toBeCloseTo(200, 4);
    expect(arcs[0]!.y).toBeCloseTo(300, 4);
    expect(arcs[0]!.r).toBeCloseTo(8, 4);
  });
});

describe("drainPresentationEffects", () => {
  it("strips renderSpiral from effects and leaves timer effects", () => {
    const drawWire = new Uint8Array(20);
    const view = new DataView(drawWire.buffer);
    view.setUint32(0, 0, true);
    view.setFloat32(4, 64, true);
    view.setFloat32(8, 64, true);

    const effects: EffectCommand[] = [
      { type: "renderSpiral", width: 64, height: 64, drawWire },
      { type: "timerStart", id: "tick", intervalMs: 16 },
    ];

    const input = encodeOutput({
      kind: "response",
      patches: [],
      effects,
      diagnostics: [],
    });

    const result = drainPresentationEffects(input);
    const output = decoder.decode(new Uint8Array(result.bytes)) as WorkerOutput;

    expect(output.kind).toBe("response");
    if (output.kind !== "response") return;
    expect(output.effects).toEqual([{ type: "timerStart", id: "tick", intervalMs: 16 }]);
    expect(result.presentation?.kind).toBe("bitmap");
    expect(result.presentation?.width).toBe(64);
  });

  it("passes through responses without render effects unchanged", () => {
    const input = encodeOutput({
      kind: "response",
      patches: [],
      effects: [{ type: "timerStop", id: "tick" }],
      diagnostics: [],
    });

    const result = drainPresentationEffects(input);
    expect(result.presentation).toBeUndefined();
    expect(result.canvasMs).toBe(0);
  });
});
