import type { WireDebugState } from "../../debug/wire-debug";
import { formatMs } from "../../worker/timings";

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

export function DebugPanel({ input }: { input: WireDebugState }) {
  const t = input.timings;

  return (
    <aside className="debug" aria-label="Wire debug">
      <h2>Timings</h2>
      <dl>
        <Row label="Last event" value={input.lastEvent} />
        <Row label="Patches" value={String(input.patchCount)} />
        {t ? (
          <>
            <Row label="Round-trip" value={formatMs(t.roundTripMs)} />
            <Row label="Wasm" value={formatMs(t.wasmMs)} />
            <Row label="Canvas2D (worker)" value={formatMs(t.canvasMs)} />
            {t.canvasProfile ? (
              <>
                <Row label="↳ Parse wire" value={formatMs(t.canvasProfile.parseMs)} />
                <Row label="↳ Clear bg" value={formatMs(t.canvasProfile.clearMs)} />
                <Row label="↳ Segment loop" value={formatMs(t.canvasProfile.loopMs)} />
                <Row
                  label="↳ setTransform"
                  value={`${formatMs(t.canvasProfile.transformMs)} (bench)`}
                />
                <Row
                  label="↳ Style/alpha"
                  value={`${formatMs(t.canvasProfile.styleMs)} (bench)`}
                />
                <Row label="↳ fillRect" value={`${formatMs(t.canvasProfile.fillMs)} (bench)`} />
                <Row label="↳ ImageBitmap" value={formatMs(t.canvasProfile.bitmapMs)} />
                <Row label="↳ Segments" value={String(t.canvasProfile.segmentCount)} />
                <Row
                  label="↳ fillStyle Δ"
                  value={String(t.canvasProfile.fillStyleChanges)}
                />
              </>
            ) : null}
            <Row label="Geometry" value={`${t.geometryKb.toFixed(1)} KB`} />
            <Row label="Decode meta" value={formatMs(t.decodeMs)} />
            <Row label="Blit (main)" value={t.blitMs != null ? formatMs(t.blitMs) : "—"} />
            <Row
              label="Blit frames"
              value={t.blitFrames != null ? String(t.blitFrames) : "—"}
            />
            <Row label="Worker total" value={formatMs(t.workerTotalMs)} />
          </>
        ) : (
          <Row label="Timings" value="—" />
        )}
      </dl>
    </aside>
  );
}
