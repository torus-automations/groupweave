import pluginNext from "@next/eslint-plugin-next";
import tseslint from "typescript-eslint";
import { reactInternalConfig } from "./react-internal.js";

/** @type {import('typescript-eslint').Config} */
export const nextJsConfig = tseslint.config(...reactInternalConfig, {
  files: ["**/*.{js,mjs,cjs,jsx,mjsx,ts,tsx,mtsx}"],
  plugins: {
    "@next/next": pluginNext,
  },
  rules: {
    ...pluginNext.configs.recommended.rules,
    ...pluginNext.configs["core-web-vitals"].rules,
    "@next/next/no-html-link-for-pages": "off",
  },
});
