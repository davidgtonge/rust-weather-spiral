#!/usr/bin/env node
/**
 * Smoke test for weather.bundle.cbor — mirrors Rust ingestion checks.
 */

import { decode } from "cbor-x";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const BUNDLE_PATH = join(__dirname, "..", "data", "weather.bundle.cbor");

const METRICS = ["cloud", "sunlight", "rain", "wind", "temperature"];
const MIN_POINTS = 8760;
const MAX_POINTS = 8784;

async function main() {
  const raw = await readFile(BUNDLE_PATH);
  const bundle = decode(raw);

  if (bundle.version !== 1) {
    throw new Error(`unexpected version ${bundle.version}`);
  }
  if (bundle.hourStep !== 3600) {
    throw new Error(`unexpected hourStep ${bundle.hourStep}`);
  }
  if (!Array.isArray(bundle.cities) || bundle.cities.length !== 4) {
    throw new Error("expected 4 cities");
  }

  for (const city of bundle.cities) {
    const { hourCount, startUnix } = city;
    if (hourCount < MIN_POINTS || hourCount > MAX_POINTS) {
      throw new Error(`${city.id}: bad hourCount ${hourCount}`);
    }

    for (const key of METRICS) {
      const blob = city[key];
      if (!(blob instanceof Uint8Array) && !Buffer.isBuffer(blob)) {
        throw new Error(`${city.id}.${key}: expected byte string`);
      }
      const bytes = blob instanceof Uint8Array ? blob : new Uint8Array(blob);
      if (bytes.byteLength !== hourCount * 4) {
        throw new Error(
          `${city.id}.${key}: expected ${hourCount * 4} bytes, got ${bytes.byteLength}`,
        );
      }
    }

    console.log(`✓ ${city.label}: ${hourCount} hours from ${startUnix}`);
  }

  console.log(`\nBundle OK (${raw.byteLength} bytes).`);
}

main().catch((err) => {
  console.error(err.message ?? err);
  process.exit(1);
});
