/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import preact from "@preact/preset-vite";

export default defineConfig({
  plugins: [preact()],
  test: {
    include: ["src/**/__tests__/**/*.test.ts"],
  },
  server: {
    port: 5174,
    strictPort: true,
  },
  worker: {
    format: "es",
  },
});
