import jsxA11y from "eslint-plugin-jsx-a11y";
import pluginReact from "eslint-plugin-react";
import pluginReactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";
import { baseConfig } from "./base.js";

/** @type {import('typescript-eslint').Config} */
export const reactInternalConfig = tseslint.config(...baseConfig, {
  files: ["**/*.{js,mjs,cjs,jsx,mjsx,ts,tsx,mtsx}"],
  plugins: {
    react: pluginReact,
    "react-hooks": pluginReactHooks,
    "jsx-a11y": jsxA11y,
  },
  rules: {
    ...pluginReact.configs.recommended.rules,
    ...pluginReactHooks.configs.recommended.rules,
    ...jsxA11y.configs.recommended.rules,
    "react/react-in-jsx-scope": "off",
    "react/prop-types": "off",
  },
  settings: {
    react: {
      version: "detect",
    },
  },
});
