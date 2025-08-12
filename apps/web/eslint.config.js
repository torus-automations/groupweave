import { nextJsConfig } from "@repo/eslint-config/next-js";
import tseslint from "typescript-eslint";

/** @type {import('typescript-eslint').Config} */
export default tseslint.config(...nextJsConfig, {
  ignores: [".next/", "dist/"],
  languageOptions: {
    parserOptions: {
      project: "./tsconfig.eslint.json",
    },
  },
});