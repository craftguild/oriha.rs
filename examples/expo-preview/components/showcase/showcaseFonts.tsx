// Font installation actions, progress and installed/available font lists.
import { useCallback, useState } from 'react';
import { Pressable, Text, View } from 'react-native';
import {
  useOrihaFonts,
  type OrihaFontInstallProgress,
  type OrihaInstallableFont,
  type OrihaInstalledFont,
} from '@craftguild/expo-oriha';
import { GaugeButton } from '@/components/gauge-button';
import { formatError } from "./showcaseRuntime";
import { styles } from "./showcaseStyles";

export type FontInstallerAction = {
  fontId: string;
  kind: 'install' | 'refresh' | 'uninstall';
};

export type FontRegistryHandle = ReturnType<typeof useOrihaFonts>;

export function useFontInstallerActions({
  installFont,
  refreshFontRegistry,
  uninstallFont,
}: Pick<FontRegistryHandle, 'installFont' | 'refreshFontRegistry' | 'uninstallFont'>) {
  const [action, setAction] = useState<FontInstallerAction | null>(null);
  const [progress, setProgress] = useState<OrihaFontInstallProgress | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  const runFontRefresh = useCallback(async () => {
    setAction({ fontId: 'registry', kind: 'refresh' });
    setProgress(null);
    setStatus(null);
    try {
      const fonts = await refreshFontRegistry();
      setStatus(`Loaded ${fonts.length} installable font packs.`);
    } catch (error: unknown) {
      setStatus(`Font registry refresh failed: ${formatError(error)}`);
    } finally {
      setAction(null);
    }
  }, [refreshFontRegistry]);

  const runFontInstall = useCallback(
    async (font: OrihaInstallableFont) => {
      setAction({ fontId: font.id, kind: 'install' });
      setProgress(null);
      setStatus(null);
      try {
        await installFont(font.id, {
          onProgress: (nextProgress) => {
            setProgress(nextProgress);
            setStatus(
              `Installing ${nextProgress.fontLabel}: ${Math.round(
                (nextProgress.overallProgress ?? 0) * 100,
              )}%`,
            );
          },
        });
        setStatus(`Installed ${font.label}.`);
      } catch (error: unknown) {
        setStatus(`Install failed for ${font.label}: ${formatError(error)}`);
      } finally {
        setAction(null);
      }
    },
    [installFont],
  );

  const runFontUninstall = useCallback(
    async (font: OrihaInstalledFont) => {
      setAction({ fontId: font.id, kind: 'uninstall' });
      setProgress(null);
      setStatus(null);
      try {
        await uninstallFont(font.id);
        setStatus(`Uninstalled ${font.label}.`);
      } catch (error: unknown) {
        setStatus(`Uninstall failed for ${font.label}: ${formatError(error)}`);
      } finally {
        setAction(null);
      }
    },
    [uninstallFont],
  );

  return {
    action,
    progress,
    runFontInstall,
    runFontRefresh,
    runFontUninstall,
    status,
  };
}

export function FontInstaller({
  action,
  error,
  isConfigured,
  installableFonts,
  installedFonts,
  isLoading,
  onInstall,
  onRefresh,
  onUninstall,
  progress,
  status,
}: {
  action: FontInstallerAction | null;
  error: unknown;
  isConfigured: boolean;
  installableFonts: OrihaInstallableFont[];
  installedFonts: OrihaInstalledFont[];
  isLoading: boolean;
  onInstall: (font: OrihaInstallableFont) => void;
  onRefresh: () => void;
  onUninstall: (font: OrihaInstalledFont) => void;
  progress: OrihaFontInstallProgress | null;
  status: string | null;
}) {
  const disabled = !isConfigured || action !== null || isLoading;

  return (
    <View style={styles.fontInstaller}>
      <View style={styles.fontInstallerHeader}>
        <View style={styles.fontInstallerTitle}>
          <Text style={styles.optionLabel}>Font Installer</Text>
          <Text style={styles.fontMeta}>Installed fonts appear in the font controls above.</Text>
          {!isConfigured ? (
            <Text style={styles.fontMeta}>
              Set EXPO_PUBLIC_FONT_REGISTRY_URL to refresh or install font packs.
            </Text>
          ) : null}
        </View>
        <SmallButton
          disabled={disabled}
          label={action?.kind === 'refresh' ? 'Refreshing' : 'Refresh'}
          onPress={onRefresh}
        />
      </View>
      {status ? <Text style={styles.fontStatus}>{status}</Text> : null}
      {progress ? (
        <Text style={styles.fontMeta}>
          {progress.fileIndex}/{progress.fileCount} files, phase: {progress.phase}
        </Text>
      ) : null}
      {error ? <Text style={styles.fontError}>{formatError(error)}</Text> : null}
      {isConfigured ? <FontList
        action={action}
        actionLabel="Install"
        emptyLabel="Refresh the registry to load installable font packs."
        fonts={installableFonts}
        onPress={onInstall}
        progress={progress}
        title={`Installable (${installableFonts.length})`}
      /> : null}
      <FontList
        action={action}
        actionLabel="Uninstall"
        emptyLabel="No installed fonts."
        fonts={installedFonts}
        onPress={onUninstall}
        title={`Installed (${installedFonts.length})`}
      />
    </View>
  );
}

export function FontList<TFont extends OrihaInstallableFont | OrihaInstalledFont>({
  action,
  actionLabel,
  emptyLabel,
  fonts,
  onPress,
  progress,
  title,
}: {
  action: FontInstallerAction | null;
  actionLabel: string;
  emptyLabel: string;
  fonts: TFont[];
  onPress: (font: TFont) => void;
  progress?: OrihaFontInstallProgress | null;
  title: string;
}) {
  return (
    <View style={styles.fontList}>
      <Text style={styles.fontListTitle}>{title}</Text>
      {fonts.length === 0 ? <Text style={styles.fontMeta}>{emptyLabel}</Text> : null}
      {fonts.map((font) => {
        const busy = action?.fontId === font.id;
        const progressValue =
          busy && action?.kind === 'install'
            ? progress?.fontId === font.id
              ? progress.overallProgress ?? 0
              : 0
            : null;
        return (
          <View key={font.id} style={styles.fontRow}>
            <View style={styles.fontRowText}>
              <Text style={styles.fontName}>{font.label}</Text>
              <Text style={styles.fontMeta}>{font.kinds.join(', ')}</Text>
            </View>
            {progressValue !== null ? (
              <GaugeButton
                label={actionLabel}
                onPress={() => onPress(font)}
                progress={progressValue}
              />
            ) : (
              <SmallButton
                disabled={action !== null}
                label={busy ? 'Working' : actionLabel}
                onPress={() => onPress(font)}
              />
            )}
          </View>
        );
      })}
    </View>
  );
}

export function SmallButton({
  disabled = false,
  label,
  onPress,
}: {
  disabled?: boolean;
  label: string;
  onPress: () => void;
}) {
  return (
    <Pressable
      accessibilityRole="button"
      accessibilityState={{ disabled }}
      disabled={disabled}
      onPress={onPress}
      style={[styles.smallButton, disabled ? styles.smallButtonDisabled : null]}
    >
      <Text style={styles.smallButtonText}>{label}</Text>
    </Pressable>
  );
}
