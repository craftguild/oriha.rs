// Font download transport, SHA-256 verification and progress accounting.
import * as Crypto from "expo-crypto";
import type { OrihaFontFile, OrihaFontInstallPhase } from "./fontRegistryModel";

export async function fetchFontPayload(
  url: string,
  onProgress?: (receivedBytes: number, totalBytes?: number) => void,
): Promise<Uint8Array> {
  if (typeof XMLHttpRequest !== "undefined") {
    return fetchFontPayloadWithXhr(url, onProgress);
  }

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Font file download failed with HTTP ${response.status}: ${url}`);
  }
  const payload = new Uint8Array(await response.arrayBuffer());
  onProgress?.(payload.byteLength, payload.byteLength);
  return payload;
}

export function fetchFontPayloadWithXhr(
  url: string,
  onProgress?: (receivedBytes: number, totalBytes?: number) => void,
): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const request = new XMLHttpRequest();
    request.open("GET", url);
    request.responseType = "arraybuffer";
    request.onprogress = (event) => {
      onProgress?.(
        event.loaded,
        event.lengthComputable ? event.total : undefined,
      );
    };
    request.onerror = () => {
      reject(new Error(`Font file download failed: ${url}`));
    };
    request.onload = () => {
      if (request.status < 200 || request.status >= 300) {
        reject(new Error(`Font file download failed with HTTP ${request.status}: ${url}`));
        return;
      }
      if (!(request.response instanceof ArrayBuffer)) {
        reject(new Error(`Font file download did not return binary data: ${url}`));
        return;
      }
      const payload = new Uint8Array(request.response);
      onProgress?.(payload.byteLength, payload.byteLength);
      resolve(payload);
    };
    request.send();
  });
}

export async function sha256Bytes(bytes: Uint8Array): Promise<string> {
  const buffer = new ArrayBuffer(bytes.byteLength);
  const data = new Uint8Array(buffer);
  data.set(bytes);
  const digest = await Crypto.digest(
    Crypto.CryptoDigestAlgorithm.SHA256,
    data,
  );
  return arrayBufferToHex(digest);
}

export function totalKnownBytes(files: readonly OrihaFontFile[]): number | undefined {
  if (!files.every((file) => typeof file.size === "number")) {
    return undefined;
  }
  return files.reduce((sum, file) => sum + (file.size ?? 0), 0);
}

export function overallProgress(
  fileIndex: number,
  fileCount: number,
  installedBytes: number,
  fileReceivedBytes: number | undefined,
  fileTotalBytes: number | undefined,
  totalBytes: number | undefined,
  phase: OrihaFontInstallPhase,
): number {
  if (totalBytes && totalBytes > 0) {
    return clampRatio((installedBytes + (fileReceivedBytes ?? 0)) / totalBytes);
  }

  const completedFiles = fileIndex - 1;
  const currentFileProgress =
    fileTotalBytes && fileTotalBytes > 0
      ? clampRatio((fileReceivedBytes ?? 0) / fileTotalBytes)
      : phase === "complete"
        ? 1
        : 0;
  return clampRatio((completedFiles + currentFileProgress) / fileCount);
}

export function progressBytes(
  fileReceivedBytes: number | undefined,
  fileTotalBytes: number | undefined,
): number {
  if (fileReceivedBytes === undefined) {
    return 0;
  }
  if (fileTotalBytes === undefined) {
    return fileReceivedBytes;
  }
  return Math.min(fileReceivedBytes, fileTotalBytes);
}

export function clampRatio(value: number): number {
  return Math.max(0, Math.min(1, value));
}

export function bytesToBase64(bytes: Uint8Array): string {
  const alphabet =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let output = "";
  for (let index = 0; index < bytes.length; index += 3) {
    const first = bytes[index];
    const second = bytes[index + 1];
    const third = bytes[index + 2];
    output += alphabet[first >> 2];
    output += alphabet[((first & 3) << 4) | ((second ?? 0) >> 4)];
    output += second === undefined ? "=" : alphabet[((second & 15) << 2) | ((third ?? 0) >> 6)];
    output += third === undefined ? "=" : alphabet[third & 63];
  }
  return output;
}

export function arrayBufferToHex(buffer: ArrayBuffer): string {
  return Array.from(new Uint8Array(buffer))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}
