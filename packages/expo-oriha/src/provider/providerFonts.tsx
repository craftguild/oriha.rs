// Font registry state, explicit operations and mount-time loading.
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  installFont,
  listInstallableFonts,
  listInstalledFonts,
  refreshFontRegistry,
  uninstallFont,
} from "../fonts/fontRegistry";
import type {
  OrihaFontRegistryConfig,
  OrihaInstallFontOptions,
  OrihaInstallableFont,
  OrihaInstalledFont,
} from "../fonts/fontRegistry";
import type { OrihaFontsState } from "./providerContext";

export function useProviderFonts(fontRegistry: OrihaFontRegistryConfig | undefined) {
  const [installedFonts, setInstalledFonts] = useState<
    OrihaInstalledFont[]
  >([]);

  const [installableFonts, setInstallableFonts] = useState<
    OrihaInstallableFont[]
  >([]);

  const [fontRegistryLoading, setFontRegistryLoading] = useState(false);

  const [fontRegistryError, setFontRegistryError] = useState<unknown>(null);

  const reloadFontRegistryState = useCallback(async () => {
    setFontRegistryLoading(true);
    setFontRegistryError(null);
    try {
      const [nextInstalledFonts, nextInstallableFonts] = await Promise.all([
        listInstalledFonts(fontRegistry),
        listInstallableFonts(fontRegistry),
      ]);
      setInstalledFonts(nextInstalledFonts);
      setInstallableFonts(nextInstallableFonts);
      return {
        installedFonts: nextInstalledFonts,
        installableFonts: nextInstallableFonts,
      };
    } catch (error: unknown) {
      setFontRegistryError(error);
      throw error;
    } finally {
      setFontRegistryLoading(false);
    }
  }, [fontRegistry]);

  const listInstalledFontsForProvider = useCallback(async () => {
    const nextInstalledFonts = await listInstalledFonts(fontRegistry);
    setInstalledFonts(nextInstalledFonts);
    return nextInstalledFonts;
  }, [fontRegistry]);

  const listInstallableFontsForProvider = useCallback(async () => {
    const nextInstallableFonts = await listInstallableFonts(fontRegistry);
    setInstallableFonts(nextInstallableFonts);
    return nextInstallableFonts;
  }, [fontRegistry]);

  const refreshFontRegistryForProvider = useCallback(async () => {
    setFontRegistryLoading(true);
    setFontRegistryError(null);
    try {
      const nextInstallableFonts = await refreshFontRegistry(fontRegistry);
      const nextInstalledFonts = await listInstalledFonts(fontRegistry);
      setInstalledFonts(nextInstalledFonts);
      setInstallableFonts(nextInstallableFonts);
      return nextInstallableFonts;
    } catch (error: unknown) {
      setFontRegistryError(error);
      throw error;
    } finally {
      setFontRegistryLoading(false);
    }
  }, [fontRegistry]);

  const installFontForProvider = useCallback(
    async (fontId: string, options?: OrihaInstallFontOptions) => {
      setFontRegistryLoading(true);
      setFontRegistryError(null);
      try {
        const installedFont = await installFont(fontId, fontRegistry, options);
        await reloadFontRegistryState();
        return installedFont;
      } catch (error: unknown) {
        setFontRegistryError(error);
        throw error;
      } finally {
        setFontRegistryLoading(false);
      }
    },
    [fontRegistry, reloadFontRegistryState],
  );

  const uninstallFontForProvider = useCallback(
    async (fontId: string) => {
      setFontRegistryLoading(true);
      setFontRegistryError(null);
      try {
        await uninstallFont(fontId, fontRegistry);
        await reloadFontRegistryState();
      } catch (error: unknown) {
        setFontRegistryError(error);
        throw error;
      } finally {
        setFontRegistryLoading(false);
      }
    },
    [fontRegistry, reloadFontRegistryState],
  );

  const fontRegistryState = useMemo<OrihaFontsState>(
    () => ({
      installedFonts,
      installableFonts,
      isLoading: fontRegistryLoading,
      error: fontRegistryError,
      listInstalledFonts: listInstalledFontsForProvider,
      listInstallableFonts: listInstallableFontsForProvider,
      refreshFontRegistry: refreshFontRegistryForProvider,
      installFont: installFontForProvider,
      uninstallFont: uninstallFontForProvider,
    }),
    [
      fontRegistryError,
      fontRegistryLoading,
      installFontForProvider,
      installableFonts,
      installedFonts,
      listInstallableFontsForProvider,
      listInstalledFontsForProvider,
      refreshFontRegistryForProvider,
      uninstallFontForProvider,
    ],
  );
  return { fontRegistryState, installedFonts, setInstalledFonts, setInstallableFonts, setFontRegistryLoading, setFontRegistryError };
}

export function useInitializeProviderFonts(fontRegistry: OrihaFontRegistryConfig | undefined, { setInstalledFonts, setInstallableFonts, setFontRegistryLoading, setFontRegistryError }: ReturnType<typeof useProviderFonts>) {
  useEffect(() => {
    let cancelled = false;
    setFontRegistryLoading(true);
    setFontRegistryError(null);
    Promise.all([listInstalledFonts(fontRegistry), listInstallableFonts(fontRegistry)])
      .then(([nextInstalledFonts, nextInstallableFonts]) => {
        if (cancelled) {
          return;
        }
        setInstalledFonts(nextInstalledFonts);
        setInstallableFonts(nextInstallableFonts);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setFontRegistryError(error);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setFontRegistryLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [
    fontRegistry,
    setFontRegistryError,
    setFontRegistryLoading,
    setInstallableFonts,
    setInstalledFonts,
  ]);
}
