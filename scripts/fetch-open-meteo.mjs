#!/usr/bin/env node
/**
 * Prefetch Open-Meteo archive hourly data for four demo cities.
 * Writes rust-weather-spiral/data/cities.json + {city}.series.json
 *
 * Usage: npm run fetch:weather
 */

import { writeFile, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, "..", "data");

const START_DATE = "2024-01-01";
const END_DATE = "2024-12-31";
const DELAY_MS = 200;

const CITIES = [
  {
    id: "bristol",
    label: "Bristol",
    latitude: 51.45,
    longitude: -2.59,
    timezone: "Europe/London",
  },
  {
    id: "ljubljana",
    label: "Ljubljana",
    latitude: 46.05,
    longitude: 14.51,
    timezone: "Europe/Ljubljana",
  },
  {
    id: "nice",
    label: "Nice",
    latitude: 43.7,
    longitude: 7.27,
    timezone: "Europe/Paris",
  },
  {
    id: "reykjavik",
    label: "Reykjavik",
    latitude: 64.15,
    longitude: -21.95,
    timezone: "Atlantic/Reykjavik",
  },
];

/** @type {Record<string, { api: string, metric: string, unit: string }>} */
const METRIC_MAP = {
  temperature: { api: "temperature_2m", metric: "temperature", unit: "°C" },
  cloud: { api: "cloud_cover", metric: "cloud", unit: "%" },
  rain: { api: "precipitation", metric: "rain", unit: "mm" },
  wind: { api: "wind_speed_10m", metric: "wind", unit: "m/s" },
  sunlight: { api: "shortwave_radiation", metric: "sunlight", unit: "W/m²" },
};

const HOURLY_VARS = [
  "temperature_2m",
  "cloud_cover",
  "precipitation",
  "wind_speed_10m",
  "shortwave_radiation",
  "sunshine_duration",
  "is_day",
].join(",");

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * @param {string} iso
 * @returns {number}
 */
function isoToUnix(iso) {
  return Math.floor(Date.parse(iso) / 1000);
}

/**
 * @param {string} cityId
 * @param {{ latitude: number, longitude: number }} city
 */
async function fetchCity(cityId, city) {
  const url = new URL("https://archive-api.open-meteo.com/v1/archive");
  url.searchParams.set("latitude", String(city.latitude));
  url.searchParams.set("longitude", String(city.longitude));
  url.searchParams.set("start_date", START_DATE);
  url.searchParams.set("end_date", END_DATE);
  url.searchParams.set("hourly", HOURLY_VARS);
  url.searchParams.set("timezone", "auto");

  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`${cityId}: HTTP ${res.status} ${res.statusText}`);
  }

  const body = await res.json();
  const hourly = body.hourly;
  if (!hourly?.time) {
    throw new Error(`${cityId}: missing hourly.time in response`);
  }

  const times = hourly.time;
  const n = times.length;

  for (const { api } of Object.values(METRIC_MAP)) {
    const values = hourly[api];
    if (!values || values.length !== n) {
      throw new Error(
        `${cityId}: ${api} length mismatch (got ${values?.length ?? 0}, expected ${n})`,
      );
    }
  }

  /** @type {Record<string, [number, number][]>} */
  const metrics = {};

  const firstUnix = isoToUnix(times[0]);
  const hourStep = 3600;

  for (const { api, metric } of Object.values(METRIC_MAP)) {
    const values = hourly[api];
    const series = [];
    for (let i = 0; i < n; i++) {
      const raw = values[i];
      if (raw === null || raw === undefined) {
        throw new Error(`${cityId}: null ${api} at index ${i} (${times[i]})`);
      }
      // Uniform hourly grid — Open-Meteo local-time series can repeat unix
      // timestamps at DST transitions; spiral layout needs strict index order.
      series.push([firstUnix + i * hourStep, Number(raw)]);
    }
    metrics[metric] = series;
  }

  const extent = [firstUnix, firstUnix + (n - 1) * hourStep];

  return {
    cityId,
    extent,
    metrics,
    pointCount: n,
  };
}

async function main() {
  await mkdir(DATA_DIR, { recursive: true });

  /** @type {typeof CITIES[number] & { file: string }[]} */
  const manifestCities = [];
  const summaries = [];

  for (let i = 0; i < CITIES.length; i++) {
    const city = CITIES[i];
    if (i > 0) await sleep(DELAY_MS);

    process.stdout.write(`Fetching ${city.label}… `);
    const series = await fetchCity(city.id, city);
    const filename = `${city.id}.series.json`;
    const outPath = join(DATA_DIR, filename);

    await writeFile(outPath, JSON.stringify(series, null, 2) + "\n");

    manifestCities.push({
      ...city,
      file: filename,
    });

    const metricCounts = Object.fromEntries(
      Object.entries(series.metrics).map(([k, v]) => [k, v.length]),
    );

    summaries.push({
      city: city.label,
      points: series.pointCount,
      extent: series.extent,
      metrics: metricCounts,
    });

    console.log(`${series.pointCount} hourly points → ${filename}`);
  }

  const manifest = {
    version: 1,
    source: "open-meteo-archive",
    startDate: START_DATE,
    endDate: END_DATE,
    cities: manifestCities,
  };

  await writeFile(
    join(DATA_DIR, "cities.json"),
    JSON.stringify(manifest, null, 2) + "\n",
  );

  console.log("\nSummary:");
  for (const s of summaries) {
    console.log(
      `  ${s.city}: ${s.points} points, extent [${s.extent[0]}, ${s.extent[1]}]`,
    );
  }
  console.log(`\nWrote ${manifestCities.length} series + cities.json to data/`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
