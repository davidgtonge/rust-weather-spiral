import {
  createViewModelStore,
  useEngineRuntime,
  useSelector,
} from "@dtonge/engine-shell";
import { useCallback, useEffect, useMemo, useRef, useState } from "preact/hooks";
import { useWireDebug } from "../debug/wire-debug";
import { weatherEffects } from "../effects";
import {
  weatherEventInput,
  weatherInitInput,
  weatherRequestFrameEvent,
  type WeatherUpdate,
} from "../worker/weather-client";
import { emptyViewModel } from "./empty-view-model";
import { loadWeatherBundle } from "../data/load-weather-bundle";

const TRANSITION_EVENTS = new Set(["citySelected", "metricSelected", "zoomSelected"]);

export function useWeatherSpiral() {
  const { debug, client, trackDispatch, reportBlit } = useWireDebug();
  const store = useMemo(() => createViewModelStore(emptyViewModel()), []);
  const vm = useSelector(store, (s) => s);
  const [bitmap, setBitmap] = useState<ImageBitmap | null>(null);
  const [bitmapSize, setBitmapSize] = useState({ width: 1024, height: 1024 });
  const [frameLoading, setFrameLoading] = useState(false);
  const frameRequested = useRef(false);
  const bitmapRef = useRef<ImageBitmap | null>(null);

  const applyVisual = useCallback((update: WeatherUpdate) => {
    const frame = update.presentation;
    if (!frame || frame.kind !== "bitmap") return;
    bitmapRef.current = frame.bitmap;
    setBitmap(frame.bitmap);
    setBitmapSize({ width: frame.width, height: frame.height });
    setFrameLoading(false);
  }, []);

  const releaseBitmap = useCallback((released: ImageBitmap) => {
    if (bitmapRef.current === released) return;
    released.close();
  }, []);

  const wrappedClient = useMemo(
    () => ({
      ...client,
      init: async (input: Parameters<typeof client.init>[0]) => {
        const update = await client.init(input);
        applyVisual(update);
        return update;
      },
      dispatch: async (input: Parameters<typeof client.dispatch>[0]) => {
        const update = await client.dispatch(input);
        applyVisual(update);
        return update;
      },
      dispose: client.dispose,
    }),
    [client, applyVisual],
  );

  const { ready, error, dispatch, init } = useEngineRuntime({
    store,
    client: wrappedClient,
    effects: weatherEffects,
    toEventInput: weatherEventInput,
  });

  useEffect(() => {
    void loadWeatherBundle()
      .then((bundle) => init(weatherInitInput(bundle)))
      .catch((err: unknown) => {
        console.error("weather bundle load failed", err);
      });
  }, [init]);

  useEffect(() => {
    if (!ready || frameRequested.current) return;
    frameRequested.current = true;
    setFrameLoading(true);
    void dispatch(weatherRequestFrameEvent());
  }, [ready, dispatch]);

  useEffect(() => {
    return () => {
      bitmapRef.current?.close();
      bitmapRef.current = null;
    };
  }, []);

  const send = useMemo(
    () =>
      trackDispatch((event) => {
        if (event.type !== "requestFrame" && event.type !== "tick") {
          setFrameLoading(!TRANSITION_EVENTS.has(event.type));
        }
        void dispatch(event);
      }),
    [trackDispatch, dispatch],
  );

  return {
    vm,
    bitmap,
    bitmapSize,
    frameLoading,
    send,
    ready,
    error,
    debug,
    onBlit: reportBlit,
    onReleaseBitmap: releaseBitmap,
  };
}
