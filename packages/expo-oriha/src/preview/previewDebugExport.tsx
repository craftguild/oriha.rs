// Owned debug HTML file export to Expo document storage.
import * as FileSystem from "expo-file-system/legacy";
import { ensureTrailingSlash, sanitizeDebugRenderedHtmlFileName } from "./orihaPreviewDebug";

export async function saveRenderedHtmlDebugFile(
  html: string,
  fileName: string,
  unavailableMessage: string,
): Promise<string> {
  const rootDirectory = FileSystem.cacheDirectory ?? FileSystem.documentDirectory;
  if (!rootDirectory) {
    throw new Error(unavailableMessage);
  }
  const directory = `${ensureTrailingSlash(rootDirectory)}oriha-debug/`;
  await FileSystem.makeDirectoryAsync(directory, { intermediates: true });
  const fileUri = `${directory}${sanitizeDebugRenderedHtmlFileName(fileName)}`;
  await FileSystem.writeAsStringAsync(fileUri, html, {
    encoding: FileSystem.EncodingType.UTF8,
  });
  return fileUri;
}
