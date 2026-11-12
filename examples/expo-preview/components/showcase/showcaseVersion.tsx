// Native bridge version loading and runtime metadata presentation.
import { useEffect, useState } from 'react';
import { Text, View } from 'react-native';
import * as Oriha from '@craftguild/expo-oriha';
import { formatError } from "./showcaseRuntime";
import { styles } from "./showcaseStyles";

export type NativeBridgeVersionInfo = {
  version: string;
  gitShortHash: string;
};

export type NativeBridgeVersionModule = typeof Oriha & {
  nativeBridgeVersionInfo?: () => Promise<NativeBridgeVersionInfo>;
};

export function useBridgeVersionInfo() {
  const [info, setInfo] = useState<NativeBridgeVersionInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const versionInfo = (Oriha as NativeBridgeVersionModule).nativeBridgeVersionInfo;

    if (!versionInfo) {
      setError('Installed package does not export nativeBridgeVersionInfo.');
      return () => {
        cancelled = true;
      };
    }

    versionInfo()
      .then((nextInfo) => {
        if (!cancelled) {
          setInfo(nextInfo);
          setError(null);
        }
      })
      .catch((nextError: unknown) => {
        if (!cancelled) {
          setError(formatError(nextError));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return { error, info };
}

export function BridgeVersionSection({
  error,
  info,
}: {
  error: string | null;
  info: NativeBridgeVersionInfo | null;
}) {
  return (
    <View style={styles.bridgeVersionSection}>
      <Text style={styles.sectionTitle}>Rust Bridge</Text>
      {error ? (
        <Text style={styles.fontError}>Version unavailable: {error}</Text>
      ) : (
        <View style={styles.bridgeVersionRows}>
          <MetadataRow label="Version" value={info?.version ?? 'Loading...'} />
          <MetadataRow label="Git hash" value={info?.gitShortHash ?? 'Loading...'} />
        </View>
      )}
    </View>
  );
}

export function MetadataRow({ label, value }: { label: string; value: string }) {
  return (
    <View style={styles.metadataRow}>
      <Text style={styles.metadataLabel}>{label}</Text>
      <Text style={styles.metadataValue}>{value}</Text>
    </View>
  );
}
