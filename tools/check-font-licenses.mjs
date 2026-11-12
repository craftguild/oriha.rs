#!/usr/bin/env node
import { resolve } from "node:path";
import { verifyLicenses } from "../packages/font-registry/scripts/registry-assets.mjs";

const manifest = verifyLicenses(resolve("packages/font-registry"));
console.log(`Verified ${manifest.fonts.length} family licenses at google/fonts ${manifest.revision}`);
