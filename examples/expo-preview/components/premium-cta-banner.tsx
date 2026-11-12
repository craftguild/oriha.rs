import { Pressable, StyleSheet, Text, View } from 'react-native';
import { ColorScheme } from '@craftguild/expo-oriha';

import { PremiumSparkleIcon } from './premium-sparkle-icon';

type PremiumCtaLocale = 'en' | 'ja' | string;

const copy = {
  en: {
    title: 'Try premium features!',
    subtitle: 'Use watermark removal, custom fonts, callouts, and diagram rendering.',
  },
  ja: {
    title: 'プレミアムフィーチャーを体験しよう！',
    subtitle: 'Watermark 削除、カスタムフォント、コールアウト、図表のレンダリングを利用しましょう。',
  },
} satisfies Record<'en' | 'ja', { subtitle: string; title: string }>;

export function PremiumCtaBanner({
  colorScheme,
  locale,
  onPress,
}: {
  colorScheme: ColorScheme;
  locale: PremiumCtaLocale;
  onPress: () => void;
}) {
  const isDark = colorScheme === ColorScheme.Dark;
  const content = locale === 'ja' ? copy.ja : copy.en;

  return (
    <Pressable
      accessibilityRole="button"
      onPress={onPress}
      style={[styles.banner, isDark ? styles.bannerDark : styles.bannerLight]}
    >
      <View style={[styles.divider, isDark ? styles.dividerDark : styles.dividerLight]} />
      <PremiumSparkleIcon />
      <View style={styles.copy}>
        <Text style={[styles.title, isDark ? styles.titleDark : styles.titleLight]} numberOfLines={1}>
          {content.title}
        </Text>
        <Text style={[styles.subtitle, isDark ? styles.subtitleDark : styles.subtitleLight]}>
          {content.subtitle}
        </Text>
      </View>
      <Text style={[styles.chevron, isDark ? styles.subtitleDark : styles.subtitleLight]}>›</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  banner: {
    alignItems: 'center',
    flexDirection: 'row',
    gap: 10,
    minHeight: 88,
    paddingHorizontal: 24,
    paddingVertical: 12,
    position: 'relative',
  },
  bannerLight: {
    backgroundColor: '#f8fafc',
  },
  bannerDark: {
    backgroundColor: '#0f172a',
  },
  divider: {
    height: StyleSheet.hairlineWidth,
    left: 0,
    position: 'absolute',
    right: 0,
    top: 0,
  },
  dividerLight: {
    backgroundColor: '#dbe3ec',
  },
  dividerDark: {
    backgroundColor: '#263449',
  },
  copy: {
    flex: 1,
    gap: 3,
    minWidth: 0,
  },
  title: {
    fontSize: 14,
    fontWeight: '700',
    lineHeight: 19,
  },
  titleLight: {
    color: '#0f172a',
  },
  titleDark: {
    color: '#f8fafc',
  },
  subtitle: {
    fontSize: 14,
    fontWeight: '400',
    lineHeight: 19,
    minHeight: 38,
  },
  subtitleLight: {
    color: '#64748b',
  },
  subtitleDark: {
    color: '#cbd5e1',
  },
  chevron: {
    fontSize: 24,
    fontWeight: '700',
  },
});
