import { useWeatherSpiral } from "./runtime/use-weather-spiral";
import { StatusScreen } from "./ui/StatusScreen";
import { WeatherView } from "./ui/WeatherView";

export function App() {
  const { vm, bitmap, bitmapSize, frameLoading, send, ready, error, debug, onBlit, onReleaseBitmap } =
    useWeatherSpiral();

  if (error) return <StatusScreen kind="error" message={error} />;
  if (!ready || !vm) return <StatusScreen kind="loading" message="Loading weather data…" />;

  return (
    <WeatherView
      vm={vm}
      bitmap={bitmap}
      bitmapWidth={bitmapSize.width}
      bitmapHeight={bitmapSize.height}
      frameLoading={frameLoading}
      send={send}
      debug={debug}
      onBlit={onBlit}
      onReleaseBitmap={onReleaseBitmap}
    />
  );
}
