#!/usr/bin/env node
/**
 * Smoke test: parse cities.json + all series files, verify point counts.
 */

import { readFile } from "node:fs/promises";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DATA_DIR = join(__dirname, "..", "data");

const EXPECTED_METRICS = ["temperature", "cloud", "rain", "wind", "sunlight"];
const MIN_POINTS = 8760;
const MAX_POINTS = 8784; // leap year hourly

/**
 * @param {unknown} value
 * @param {string} label
 */
function assertNumber(value, label) {
  if (typeof value !== "number" || Number.isNaN(value)) {
    throw new Error(`${label}: expected number, got ${value}`);
  }
}

/**
 * @param {[number, number][]} series
 * @param {string} label
 */
function validateSeries(series, label) {
  if (!Array.isArray(series) || series.length === 0) {
    throw new Error(`${label}: empty or missing series`);
  }
  if (series.length < MIN_POINTS || series.length > MAX_POINTS) {
    throw new Error(
      `${label}: expected ${MIN_POINTS}–${MAX_POINTS} points, got ${series.length}`,
    );
  }
  for (let i = 0; i < series.length; i++) {
    const pair = series[i];
    if (!Array.isArray(pair) || pair.length !== 2) {
      throw new Error(`${label}[${i}]: expected [unix, value]`);
    }
    assertNumber(pair[0], `${label}[${i}].time`);
    assertNumber(pair[1], `${label}[${i}].value`);
    if (i > 0 && pair[0] <= series[i - 1][0]) {
      throw new Error(`${label}[${i}]: timestamps not strictly increasing`);
    }
  }
}

async function main() {
  const manifestRaw = await readFile(join(DATA_DIR, "cities.json"), "utf8");
  const manifest = JSON.parse(manifestRaw);

  if (manifest.version !== 1) {
    throw new Error(`cities.json: unexpected version ${manifest.version}`);
  }
  if (!Array.isArray(manifest.cities) || manifest.cities.length !== 4) {
    throw new Error(`cities.json: expected 4 cities`);
  }

  for (const city of manifest.cities) {
    const seriesPath = join(DATA_DIR, city.file);
    const seriesRaw = await readFile(seriesPath, "utf8");
    const series = JSON.parse(seriesRaw);

    if (series.cityId !== city.id) {
      throw new Error(`${city.file}: cityId mismatch (${series.cityId} vs ${city.id})`);
    }

    for (const metric of EXPECTED_METRICS) {
      validateSeries(series.metrics[metric], `${city.id}.${metric}`);
    }

    console.log(
      `✓ ${city.label}: ${series.metrics.temperature.length} points, extent [${series.extent[0]}, ${series.extent[1]}]`,
    );
  }

  console.log(`\nAll ${manifest.cities.length} cities validated.`);
}

main().catch((err) => {
  console.error(err.message ?? err);
  process.exit(1);
});
