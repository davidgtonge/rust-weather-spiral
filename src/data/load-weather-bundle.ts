import weatherBundleUrl from "../../data/weather.bundle.cbor?url";

let cached: Uint8Array | null = null;

/** Fetches the compiled weather CBOR bundle (built by `npm run build:weather-cbor`). */
export async function loadWeatherBundle(): Promise<Uint8Array> {
  if (cached) return cached;

  const res = await fetch(weatherBundleUrl);
  if (!res.ok) {
    throw new Error(`weather bundle fetch failed: ${res.status} ${res.statusText}`);
  }

  cached = new Uint8Array(await res.arrayBuffer());
  return cached;
}
