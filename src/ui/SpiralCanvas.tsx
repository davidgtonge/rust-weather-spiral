import { useEffect, useRef } from "preact/hooks";
import type { BlitReport } from "../worker/timings";

const TRANSITION_MS = 360;

function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3;
}

function prefersReducedMotion(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

type SpiralCanvasProps = {
  bitmap: ImageBitmap | null;
  loading?: boolean;
  displayWidth: number;
  displayHeight: number;
  bitmapWidth: number;
  bitmapHeight: number;
  onBlit?: (report: BlitReport) => void;
  onReleaseBitmap?: (bitmap: ImageBitmap) => void;
};

export function SpiralCanvas({
  bitmap,
  loading = false,
  displayWidth,
  displayHeight,
  bitmapWidth,
  bitmapHeight,
  onBlit,
  onReleaseBitmap,
}: SpiralCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const heldRef = useRef<ImageBitmap | null>(null);
  const rafRef = useRef(0);

  useEffect(() => {
    return () => {
      cancelAnimationFrame(rafRef.current);
      if (heldRef.current) {
        onReleaseBitmap?.(heldRef.current);
        heldRef.current = null;
      }
    };
  }, [onReleaseBitmap]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !bitmap) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    cancelAnimationFrame(rafRef.current);

    const prev = heldRef.current;
    heldRef.current = bitmap;

    const blitInstant = () => {
      const blitStart = performance.now();
      ctx.clearRect(0, 0, bitmapWidth, bitmapHeight);
      ctx.globalAlpha = 1;
      ctx.drawImage(bitmap, 0, 0);
      onBlit?.({ ms: performance.now() - blitStart, frames: 1 });
      if (prev && prev !== bitmap) {
        onReleaseBitmap?.(prev);
      }
    };

    if (!prev || prev === bitmap || prefersReducedMotion()) {
      blitInstant();
      return;
    }

    const start = performance.now();
    let frames = 0;
    const tick = (now: number) => {
      frames++;
      const t = easeOutCubic(Math.min(1, (now - start) / TRANSITION_MS));
      ctx.clearRect(0, 0, bitmapWidth, bitmapHeight);
      ctx.globalAlpha = 1;
      ctx.drawImage(prev, 0, 0);
      ctx.globalAlpha = t;
      ctx.drawImage(bitmap, 0, 0);
      ctx.globalAlpha = 1;

      if (t < 1) {
        rafRef.current = requestAnimationFrame(tick);
      } else {
        onBlit?.({ ms: performance.now() - start, frames });
        onReleaseBitmap?.(prev);
      }
    };

    rafRef.current = requestAnimationFrame(tick);
  }, [bitmap, bitmapWidth, bitmapHeight, onBlit, onReleaseBitmap]);

  return (
    <div
      className="spiral-canvas-wrap"
      style={{ width: displayWidth, height: displayHeight }}
      data-loading={loading ? "true" : "false"}
    >
      <canvas
        ref={canvasRef}
        className="spiral-canvas"
        width={bitmapWidth}
        height={bitmapHeight}
        style={{ width: displayWidth, height: displayHeight }}
        aria-label="Weather spiral visualisation"
        aria-busy={loading}
      />
      {loading && !bitmap ? <p className="spiral-loading">Rendering spiral…</p> : null}
    </div>
  );
}
