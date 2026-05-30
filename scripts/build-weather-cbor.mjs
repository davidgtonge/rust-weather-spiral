#!/usr/bin/env node
/**
 * Compile human-readable JSON series into a compact Wasm-ready CBOR bundle.
 *
 * Wire shape (one decode in Rust):
 *   { version, hourStep, cities: [{ id, label, startUnix, hourCount,
 *     cloud, sunlight, rain, wind, temperature }] }
 *
 * Each metric is a CBOR byte string of little-endian f32 values (no timestamps —
 * uniform hourly grid: time(i) = startUnix + i * hourStep).
 */

import { encode } from "cbor-x";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, "..", "data");

const HOUR_STEP = 3600;
const METRICS = ["cloud", "sunlight", "rain", "wind", "temperature"];

/**
 * @param {[number, number][]} pairs
 */
function valuesToF32Bytes(pairs) {
  const buf = Buffer.alloc(pairs.length * 4);
  for (let i = 0; i < pairs.length; i++) {
    buf.writeFloatLE(pairs[i][1], i * 4);
  }
  return buf;
}

async function main() {
  const manifestRaw = await readFile(join(DATA_DIR, "cities.json"), "utf8");
  const manifest = JSON.parse(manifestRaw);

  /** @type {Record<string, unknown>[]} */
  const cities = [];

  for (const entry of manifest.cities) {
    const seriesPath = join(DATA_DIR, entry.file);
    const series = JSON.parse(await readFile(seriesPath, "utf8"));

    const hourCount = series.metrics.temperature.length;
    const startUnix = series.extent[0];

    /** @type {Record<string, unknown>} */
    const city = {
      id: entry.id,
      label: entry.label,
      startUnix,
      hourCount,
    };

    for (const key of METRICS) {
      const pairs = series.metrics[key];
      if (!pairs || pairs.length !== hourCount) {
        throw new Error(`${entry.id}: ${key} length mismatch`);
      }
      city[key] = valuesToF32Bytes(pairs);
    }

    cities.push(city);
  }

  const bundle = {
    version: 1,
    hourStep: HOUR_STEP,
    cities,
  };

  const outPath = join(DATA_DIR, "weather.bundle.cbor");
  const bytes = encode(bundle);
  await mkdir(DATA_DIR, { recursive: true });
  await writeFile(outPath, bytes);

  const mb = bytes.length / (1024 * 1024);
  console.log(
    `Wrote ${outPath} — ${bytes.length} bytes (${mb.toFixed(2)} MB), ${cities.length} cities`,
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
