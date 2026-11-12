import { defineConfig } from "tsdown";
import { bundleEvidence } from "./scripts/bundle-evidence.mjs";

export default defineConfig({
  entry: ["src/index.ts"],
  dts: true,
  plugins: [bundleEvidence()],
  deps: {
    neverBundle: [
      "expo",
      "expo-crypto",
      "expo-file-system",
      "expo-file-system/legacy",
      "expo-modules-core",
      "expo-print",
      "expo-sharing",
      "react",
      "react-native",
      "react-native-svg",
      "react-native-webview",
    ],
  },
  format: {
    esm: {
      target: ["es2020"],
    },
    cjs: {
      target: ["node12"],
    },
  },
});
