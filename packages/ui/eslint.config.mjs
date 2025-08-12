import { reactInternalConfig } from "@repo/eslint-config/react-internal";
import tseslint from "typescript-eslint";

/** @type {import('typescript-eslint').Config} */
export default tseslint.config(...reactInternalConfig, {
  ignores: ["dist/"],
  languageOptions: {
    parserOptions: {
      project: "./tsconfig.eslint.json",
    },
  },
});