import { useState } from 'react';
import { Pressable, StyleSheet, Text, View, type LayoutChangeEvent } from 'react-native';

export function GaugeButton({
  disabled = false,
  label,
  onPress,
  progress,
}: {
  disabled?: boolean;
  label: string;
  onPress: () => void;
  progress: number | null;
}) {
  const [buttonWidth, setButtonWidth] = useState(0);
  const isProgressVisible = progress !== null;
  const normalizedProgress = isProgressVisible ? clamp(progress, 0, 1) : 0;
  const progressLabel = `${Math.round(normalizedProgress * 100)}%`;
  const progressWidth = buttonWidth * normalizedProgress;
  const isDisabled = disabled || isProgressVisible;
  const handleLayout = (event: LayoutChangeEvent) => {
    setButtonWidth(event.nativeEvent.layout.width);
  };

  return (
    <Pressable
      accessibilityRole="button"
      accessibilityState={{ disabled: isDisabled, busy: isProgressVisible }}
      disabled={isDisabled}
      onLayout={handleLayout}
      onPress={onPress}
      style={[styles.button, isDisabled ? styles.buttonDisabled : null]}
    >
      {isProgressVisible ? (
        <View pointerEvents="none" style={[styles.fill, { width: progressWidth }]} />
      ) : null}
      <Text style={styles.text}>{isProgressVisible ? progressLabel : label}</Text>
    </Pressable>
  );
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

const styles = StyleSheet.create({
  button: {
    alignItems: 'center',
    backgroundColor: '#0f172a',
    borderRadius: 6,
    justifyContent: 'center',
    minHeight: 32,
    minWidth: 82,
    overflow: 'hidden',
    paddingHorizontal: 10,
  },
  buttonDisabled: {
    opacity: 0.86,
  },
  fill: {
    backgroundColor: '#2563eb',
    bottom: 0,
    left: 0,
    position: 'absolute',
    top: 0,
  },
  text: {
    color: '#ffffff',
    fontSize: 12,
    fontWeight: '800',
    zIndex: 1,
  },
});
