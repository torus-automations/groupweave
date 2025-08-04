import eslint from "@eslint/js";
import { fixupPluginRules } from "@eslint/compat";
import importPlugin from "eslint-plugin-i";
import turboPlugin from "eslint-plugin-turbo";
import tseslint from "typescript-eslint";

/** @type {import('typescript-eslint').Config} */
export const baseConfig = tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    plugins: {
      i: fixupPluginRules(importPlugin),
      turbo: turboPlugin,
    },
    rules: {
      "turbo/no-undeclared-env-vars": "warn",
      "i/no-unresolved": "error",
      "i/no-unused-modules": [
        "warn",
        {
          unusedExports: true,
        },
      ],
    },
  },
  {
    ignores: ["dist/**"],
  },
);
