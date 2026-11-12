// Toast animation, overlay placement and visual variants.
import React, { useEffect, useRef } from "react";
import { Animated, Text } from "react-native";
import type { StyleProp, ViewStyle } from "react-native";
import Svg, { Path } from "react-native-svg";
import { OrihaOverlayPlacement, OrihaToastPosition, OrihaToastVariant } from "./OrihaEnums";
import { HEROICON_SOLID_DEFINITIONS, type HeroiconSolidName } from "../../generated/generatedHeroicons";
import type { ToastEntry } from "./providerContext";
import { styles } from "./providerStyles";

export const TOAST_POSITIONS = [
  OrihaToastPosition.Top,
  OrihaToastPosition.Center,
  OrihaToastPosition.Bottom,
] as const;

export type ToastVariantStyle = {
  backgroundColor: string;
  iconColor: string;
  textColor: string;
  icon: HeroiconSolidName;
};

export function OrihaToast({
  toast,
  onDismiss,
}: {
  toast: ToastEntry;
  onDismiss: (id: string) => void;
}) {
  const progress = useRef(new Animated.Value(0)).current;
  const variantStyle = toastVariantStyle(toast.variant);

  useEffect(() => {
    Animated.timing(progress, {
      toValue: 1,
      duration: 180,
      useNativeDriver: true,
    }).start();

    const timer = setTimeout(() => {
      Animated.timing(progress, {
        toValue: 0,
        duration: 160,
        useNativeDriver: true,
      }).start(({ finished }) => {
        if (finished) {
          onDismiss(toast.id);
        }
      });
    }, toast.durationMs);

    return () => {
      clearTimeout(timer);
    };
  }, [onDismiss, progress, toast.durationMs, toast.id]);

  const translateY = progress.interpolate({
    inputRange: [0, 1],
    outputRange: [
      toast.position === OrihaToastPosition.Top ? -24 : 24,
      0,
    ],
  });

  return (
    <Animated.View
      pointerEvents="none"
      style={[
        styles.toast,
        { backgroundColor: variantStyle.backgroundColor },
        {
          opacity: progress,
          transform: [{ translateY }],
        },
      ]}
    >
      <Heroicon
        color={variantStyle.iconColor}
        name={variantStyle.icon}
        style={styles.toastIcon}
      />
      <Text style={[styles.toastText, { color: variantStyle.textColor }]}>
        {toast.message}
      </Text>
    </Animated.View>
  );
}

export function Heroicon({
  color,
  name,
  style,
}: {
  color: string;
  name: HeroiconSolidName;
  style?: StyleProp<ViewStyle>;
}) {
  const icon = HEROICON_SOLID_DEFINITIONS[name];

  return (
    <Svg
      fill={color}
      height={20}
      style={style}
      viewBox={icon.viewBox}
      width={20}
    >
      {icon.paths.map((path, index) => (
        <Path key={`${name}-${index}`} {...path} />
      ))}
    </Svg>
  );
}

export function toastVariantStyle(
  variant: OrihaToastVariant,
): ToastVariantStyle {
  switch (variant) {
    case OrihaToastVariant.Info:
      return {
        backgroundColor: "#0284c7",
        icon: "information-circle",
        iconColor: "#e0f2fe",
        textColor: "#ffffff",
      };
    case OrihaToastVariant.Note:
      return {
        backgroundColor: "#4f46e5",
        icon: "pencil",
        iconColor: "#e0e7ff",
        textColor: "#ffffff",
      };
    case OrihaToastVariant.Success:
      return {
        backgroundColor: "#059669",
        icon: "check-circle",
        iconColor: "#d1fae5",
        textColor: "#ffffff",
      };
    case OrihaToastVariant.Warn:
      return {
        backgroundColor: "#f59e0b",
        icon: "exclamation-triangle",
        iconColor: "#78350f",
        textColor: "#111827",
      };
    case OrihaToastVariant.Err:
      return {
        backgroundColor: "#e11d48",
        icon: "exclamation-circle",
        iconColor: "#ffe4e6",
        textColor: "#ffffff",
      };
    case OrihaToastVariant.Neutral:
    default:
      return {
        backgroundColor: "#0f172a",
        icon: "megaphone",
        iconColor: "#cbd5e1",
        textColor: "#ffffff",
      };
  }
}

export function overlayPlacementStyle(
  placement: OrihaOverlayPlacement,
): StyleProp<ViewStyle> {
  switch (placement) {
    case OrihaOverlayPlacement.Top:
      return styles.overlayTop;
    case OrihaOverlayPlacement.Bottom:
      return styles.overlayBottom;
    case OrihaOverlayPlacement.Fill:
      return styles.overlayFill;
    case OrihaOverlayPlacement.Center:
    default:
      return styles.overlayCenter;
  }
}

export function toastPositionStyle(
  position: OrihaToastPosition,
): StyleProp<ViewStyle> {
  switch (position) {
    case OrihaToastPosition.Top:
      return styles.toastSlotTop;
    case OrihaToastPosition.Center:
      return styles.toastSlotCenter;
    case OrihaToastPosition.Bottom:
    default:
      return styles.toastSlotBottom;
  }
}
