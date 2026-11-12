// Independent gauge, premium-banner and router-tab component examples.
import { useCallback, useEffect, useState } from 'react';
import { Text, View } from 'react-native';
import { ColorScheme } from '@craftguild/expo-oriha';
import { GaugeButton } from '@/components/gauge-button';
import { styles } from "./showcaseStyles";

export const GAUGE_BUTTON_DEMO_DURATION_MS = 5000;

export function ComponentsSection() {
  const [demoStartedAt, setDemoStartedAt] = useState<number | null>(null);
  const [demoProgress, setDemoProgress] = useState<number | null>(null);

  useEffect(() => {
    if (demoStartedAt === null) {
      return;
    }

    let resetTimeout: ReturnType<typeof setTimeout> | null = null;
    const interval = setInterval(() => {
      const nextProgress = Math.min(
        (Date.now() - demoStartedAt) / GAUGE_BUTTON_DEMO_DURATION_MS,
        1,
      );
      setDemoProgress(nextProgress);

      if (nextProgress >= 1) {
        clearInterval(interval);
        resetTimeout = setTimeout(() => {
          setDemoStartedAt(null);
          setDemoProgress(null);
        }, 200);
      }
    }, 50);

    return () => {
      clearInterval(interval);
      if (resetTimeout) {
        clearTimeout(resetTimeout);
      }
    };
  }, [demoStartedAt]);

  const startGaugeDemo = useCallback(() => {
    setDemoStartedAt(Date.now());
    setDemoProgress(0);
  }, []);

  return (
    <View style={styles.componentsSection}>
      <Text style={styles.sectionTitle}>コンポーネント</Text>
      <View style={styles.componentRow}>
        <View style={styles.componentText}>
          <Text style={styles.componentName}>ゲージ付きボタン</Text>
          <Text style={styles.fontMeta}>Font download progress</Text>
        </View>
        <GaugeButton label="ボタン" onPress={startGaugeDemo} progress={demoProgress} />
      </View>
    </View>
  );
}

export function RouterTabsPreview({ colorScheme }: { colorScheme: ColorScheme }) {
  const isDark = colorScheme === ColorScheme.Dark;
  const tabs = ['Home', 'Preview', 'Settings'] as const;

  return (
    <View style={[styles.routerTabsPreview, isDark ? styles.routerTabsPreviewDark : null]}>
      {tabs.map((tab) => {
        const selected = tab === 'Preview';
        return (
          <View key={tab} style={styles.routerTabItem}>
            <View
              style={[
                styles.routerTabIndicator,
                selected ? styles.routerTabIndicatorSelected : null,
              ]}
            />
            <Text
              style={[
                styles.routerTabLabel,
                isDark ? styles.routerTabLabelDark : styles.routerTabLabelLight,
                selected ? styles.routerTabLabelSelected : null,
              ]}
            >
              {tab}
            </Text>
          </View>
        );
      })}
    </View>
  );
}
