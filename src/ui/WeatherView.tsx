import type { WireDebugState } from "../debug/wire-debug";
import type { BlitReport } from "../worker/timings";
import type { AppEvent, ViewModel } from "../protocol/types";
import { CityPicker } from "./components/CityPicker";
import { DebugPanel } from "./components/DebugPanel";
import { Legend } from "./components/Legend";
import { MetricTabs } from "./components/MetricTabs";
import { ViewModePicker } from "./components/ViewModePicker";
import { ZoomControl } from "./components/ZoomControl";
import { SpiralCanvas } from "./SpiralCanvas";

type WeatherViewProps = {
  vm: ViewModel;
  bitmap: ImageBitmap | null;
  bitmapWidth: number;
  bitmapHeight: number;
  frameLoading: boolean;
  send: (event: AppEvent) => void;
  debug: WireDebugState;
  onBlit: (report: BlitReport) => void;
  onReleaseBitmap: (bitmap: ImageBitmap) => void;
};

export function WeatherView({
  vm,
  bitmap,
  bitmapWidth,
  bitmapHeight,
  frameLoading,
  send,
  debug,
  onBlit,
  onReleaseBitmap,
}: WeatherViewProps) {
  const displaySize = Math.min(640, typeof window !== "undefined" ? window.innerWidth - 48 : 640);

  return (
    <main className="weather-app">
      <header className="weather-header">
        <div>
          <h1>Weather Spiral</h1>
          <p className="subtitle">
            {vm.cityLabel} · {vm.viewModeLabel} · {vm.showMetricTabs ? `${vm.metricLabel} · ` : ""}
            {vm.zoomLabel}
          </p>
        </div>
        <CityPicker cities={vm.cities} selectedId={vm.selectedCityId} onSelect={send} />
      </header>

      <div className="weather-layout">
        <section className="spiral-stage">
          <SpiralCanvas
            bitmap={bitmap}
            loading={frameLoading}
            displayWidth={displaySize}
            displayHeight={displaySize}
            bitmapWidth={bitmapWidth}
            bitmapHeight={bitmapHeight}
            onBlit={onBlit}
            onReleaseBitmap={onReleaseBitmap}
          />
        </section>

        <aside className="sidebar">
          <ViewModePicker
            selected={vm.selectedViewMode}
            description={vm.viewModeDescription}
            onSelect={send}
          />
          {vm.showMetricTabs ? (
            <MetricTabs selected={vm.selectedMetric} onSelect={send} />
          ) : null}
          <ZoomControl selected={vm.selectedZoom} onSelect={send} />
          <Legend vm={vm} />
          <p className="attribution">
            Weather data by{" "}
            <a href="https://open-meteo.com/" target="_blank" rel="noreferrer">
              Open-Meteo
            </a>
            . Historical archive API.
          </p>
          {import.meta.env.DEV ? <DebugPanel input={debug} /> : null}
        </aside>
      </div>
    </main>
  );
}
