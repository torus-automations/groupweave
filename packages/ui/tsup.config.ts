import { defineConfig, type Options } from "tsup";
import pathAlias from 'esbuild-plugin-path-alias';

export default defineConfig((options: Options) => ({
  entry: ["src/**/*.tsx", "src/**/*.ts"],
  format: ["esm", "cjs"],
  dts: true,
  minify: true,
  external: ["react"],
  esbuildPlugins: [pathAlias({ alias: { "@": "./src" } })],
  ...options,
}));