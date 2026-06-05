import { useCallback, useMemo, useState } from "preact/hooks";
import type { AppEvent, WorkerOutput } from "../protocol/types";
import { createWeatherWorkerClient } from "../worker/weather-client";
import { logTimings, type BlitReport, type FrameTimings } from "../worker/timings";

export type WireDebugState = {
  lastEvent: string;
  patchCount: number;
  effectCount: number;
  timings: FrameTimings | null;
  raw: WorkerOutput | null;
};

const EMPTY_DEBUG: WireDebugState = {
  lastEvent: "—",
  patchCount: 0,
  effectCount: 0,
  timings: null,
  raw: null,
};

export function useWireDebug() {
  const [debug, setDebug] = useState<WireDebugState>(EMPTY_DEBUG);

  const onTimings = useCallback(
    (label: string, timings: FrameTimings, output: WorkerOutput) => {
      setDebug((prev) => ({
        ...prev,
        lastEvent: label,
        timings,
        raw: output,
        patchCount: output.kind === "response" ? output.patches.length : prev.patchCount,
        effectCount:
          output.kind === "response" || output.kind === "initialized"
            ? output.effects.length
            : prev.effectCount,
      }));

      if (timings.geometryKb === 0) {
        logTimings(label, timings);
      }
    },
    [],
  );

  const client = useMemo(() => createWeatherWorkerClient(onTimings), [onTimings]);

  const trackDispatch = useCallback(
    (dispatch: (event: AppEvent) => void) =>
      (event: AppEvent) => {
        setDebug((prev) => ({ ...prev, lastEvent: event.type }));
        dispatch(event);
      },
    [],
  );

  const reportBlit = useCallback(({ ms, frames }: BlitReport) => {
    setDebug((prev) => {
      if (!prev.timings) return prev;
      const timings = { ...prev.timings, blitMs: ms, blitFrames: frames };
      logTimings(prev.lastEvent, timings);
      return { ...prev, timings };
    });
  }, []);

  return { debug, client, trackDispatch, reportBlit };
}
