// Use the repository's locked Expo ESLint toolchain for all npm implementation.
import { createRequire } from "node:module";

const require = createRequire(new URL("./examples/expo-preview/package.json", import.meta.url));
const { defineConfig } = require("eslint/config");
const expo = require("eslint-config-expo/flat");
const globals = require("globals");

export default defineConfig([
  {
    ignores: [
      "**/node_modules/**", "**/dist/**", "**/.astro/**", "**/.vercel/**",
      "**/generated/**", ".craftguild/**", ".private/**", "vendor/**",
    ],
  },
  expo,
  {
    settings: { react: { version: require("react/package.json").version } },
  },
  {
    files: ["**/*.{js,mjs,cjs,mts}", "packages/font-registry/**/*.ts"],
    languageOptions: { globals: globals.node },
    // Node tools inspect environment keys at runtime; Expo's inlining rule
    // remains active for Expo application and library source.
    rules: { "expo/no-dynamic-env-var": "off" },
  },
  {
    files: ["packages/font-registry/src/middleware.ts"],
    // Astro supplies this documented virtual module during its build.
    settings: { "import/core-modules": ["astro:middleware"] },
  },
]);
