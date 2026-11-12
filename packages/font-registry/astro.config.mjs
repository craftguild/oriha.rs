import { defineConfig } from "astro/config";
import vercel from "@astrojs/vercel";
import tailwindcss from "@tailwindcss/vite";
import { bundleEvidence } from "./scripts/bundle-evidence.mjs";

export default defineConfig({
  output: "server",
  compressHTML: true,
  publicDir: "./static",
  // Registry pages serve fonts; they do not transform images or ship host libvips.
  image: { service: { entrypoint: "astro/assets/services/noop" } },
  adapter: vercel({
    includeFiles: ["generated"],
  }),
  vite: {
    plugins: [tailwindcss(), bundleEvidence()],
  },
});
