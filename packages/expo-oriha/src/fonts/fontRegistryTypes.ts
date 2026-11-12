export type FontRegistryFontKind = "heading" | "body" | "code";

export type FontRegistrySourcePolicy = {
  allowTextParameter: false;
  allowPerDocumentSubset: false;
};

export type FontRegistryConfig = {
  version: string | number;
  source: {
    provider: "google-fonts-css2";
    baseUrl: string;
    display?: string;
    policy: FontRegistrySourcePolicy;
  };
  fonts: FontRegistryConfigFont[];
};

export type FontRegistryConfigFont = {
  id: string;
  label: string;
  family: string;
  kinds: FontRegistryFontKind[];
  source: {
    specimenUrl?: string;
    css2Family: string;
  };
};

export type FontRegistryFontFamilySpec = {
  family: string;
  fallbacks?: string[];
};

export type FontRegistryFontFile = {
  url: string;
  fileName?: string;
  sha256: string;
  weight?: string | number;
  style?: string;
  format?: string;
  unicodeRange?: string | null;
  size?: number;
};

export type FontRegistryFont = {
  id: string;
  label: string;
  family: string;
  kinds: FontRegistryFontKind[];
  fontFamilySpec: FontRegistryFontFamilySpec;
  version: string;
  files: FontRegistryFontFile[];
  description?: string;
};

export type FontRegistryManifest = {
  version: string | number;
  generatedAt: string;
  source: {
    provider: "google-fonts-css2";
    policy: FontRegistrySourcePolicy;
  };
  fonts: FontRegistryFont[];
};
