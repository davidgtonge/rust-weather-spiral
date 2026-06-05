/**
 * Draw-command wire from Rust `build_geometry_wire`.
 *
 * Header (20 B): count u32, canvasW f32, canvasH f32, playhead u32, viewMode u32
 * Body: count × 48 B draw commands (op, flags, rgba, transforms, params)
 */

export const GEOMETRY_HEADER = 20;
export const BYTES_PER_COMMAND = 48;
/** Legacy alias — wire is draw commands, not segment SoA. */
export const BYTES_PER_SEGMENT = BYTES_PER_COMMAND;

export const OP_FILL_RECT = 0;
export const OP_STROKE_LINE = 1;
export const OP_STROKE_ARC = 2;
export const OP_FILL_CIRCLE = 3;
export const OP_FILL_ELLIPSE = 4;
export const OP_STROKE_RECT = 5;
export const OP_BEGIN_CLIP = 6;
export const OP_END_CLIP = 7;

export const FL_CANVAS_SPACE = 1;

export type ViewModeWire =
  | "metric"
  | "tapestry"
  | "ribbon"
  | "condition"
  | "glyphs"
  | "mandala"
  | "fingerprint"
  | "daylight";

export type DrawCommand = {
  op: number;
  flags: number;
  r: number;
  g: number;
  b: number;
  a: number;
  lineWidth: number;
  cos: number;
  sin: number;
  tx: number;
  ty: number;
  p0: number;
  p1: number;
  p2: number;
  p3: number;
  p4: number;
};

export type DrawWire = {
  count: number;
  canvasW: number;
  canvasH: number;
  playheadIndex: number;
  viewMode: ViewModeWire;
  commands: DrawCommand[];
};

/** @deprecated Use DrawWire — kept for callers that referenced GeometrySoA. */
export type GeometrySoA = DrawWire;

export const FLAG_STORM = 1;
export const FLAG_BRIGHT = 2;

export function wireByteLength(count: number): number {
  return GEOMETRY_HEADER + count * BYTES_PER_COMMAND;
}

function viewModeFromWire(value: number): ViewModeWire {
  switch (value) {
    case 1:
      return "tapestry";
    case 2:
      return "ribbon";
    case 3:
      return "condition";
    case 4:
      return "glyphs";
    case 5:
      return "mandala";
    case 6:
      return "fingerprint";
    case 7:
      return "daylight";
    default:
      return "metric";
  }
}

function readCommand(view: DataView, offset: number): DrawCommand {
  return {
    op: view.getUint8(offset),
    flags: view.getUint8(offset + 1),
    r: view.getUint8(offset + 2),
    g: view.getUint8(offset + 3),
    b: view.getUint8(offset + 4),
    a: view.getUint8(offset + 5),
    lineWidth: view.getFloat32(offset + 8, true),
    cos: view.getFloat32(offset + 12, true),
    sin: view.getFloat32(offset + 16, true),
    tx: view.getFloat32(offset + 20, true),
    ty: view.getFloat32(offset + 24, true),
    p0: view.getFloat32(offset + 28, true),
    p1: view.getFloat32(offset + 32, true),
    p2: view.getFloat32(offset + 36, true),
    p3: view.getFloat32(offset + 40, true),
    p4: view.getFloat32(offset + 44, true),
  };
}

/** Parse draw-command wire from Rust. */
export function parseGeometryWire(buffer: ArrayBuffer): DrawWire {
  const view = new DataView(buffer);
  const count = view.getUint32(0, true);
  const commands: DrawCommand[] = new Array(count);
  let offset = GEOMETRY_HEADER;
  for (let i = 0; i < count; i++) {
    commands[i] = readCommand(view, offset);
    offset += BYTES_PER_COMMAND;
  }

  return {
    count,
    canvasW: view.getFloat32(4, true),
    canvasH: view.getFloat32(8, true),
    playheadIndex: view.getUint32(12, true),
    viewMode: viewModeFromWire(view.getUint32(16, true)),
    commands,
  };
}
