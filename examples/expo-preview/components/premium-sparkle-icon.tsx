import { useEffect, useState } from 'react';
import Svg, { Defs, G, LinearGradient, Path, Stop } from 'react-native-svg';

const ANIMATION_DURATION_MS = 12000;
const FRAME_INTERVAL_MS = 80;
const HUE_SPAN = 0.28;
const STOP_COUNT = 12;

export function PremiumSparkleIcon({
  frameSize = 24,
}: {
  frameSize?: number;
}) {
  const [phase, setPhase] = useState(0);

  useEffect(() => {
    const startedAt = Date.now();
    const interval = setInterval(() => {
      setPhase(((Date.now() - startedAt) % ANIMATION_DURATION_MS) / ANIMATION_DURATION_MS);
    }, FRAME_INTERVAL_MS);

    return () => {
      clearInterval(interval);
    };
  }, []);

  const stops = Array.from({ length: STOP_COUNT + 1 }, (_, index) => {
    const offset = index / STOP_COUNT;
    const hue = (phase + offset * HUE_SPAN) % 1;
    return {
      color: hslToHex(hue, 0.66, 0.58),
      offset,
    };
  });

  return (
    <Svg
      accessibilityElementsHidden
      importantForAccessibility="no-hide-descendants"
      height={frameSize}
      viewBox="0 0 24 24"
      width={frameSize}
    >
      <Defs>
        <LinearGradient id="premiumSparkleGradient" x1="0" x2="1" y1="0" y2="1">
          {stops.map((stop) => (
            <Stop
              key={`${stop.offset}-${stop.color}`}
              offset={`${Math.round(stop.offset * 100)}%`}
              stopColor={stop.color}
            />
          ))}
        </LinearGradient>
      </Defs>
      <G fill="url(#premiumSparkleGradient)">
        <Path d="M12 2.4l1.95 5.16L19.2 9.5l-5.25 1.94L12 16.6l-1.95-5.16L4.8 9.5l5.25-1.94L12 2.4z" />
        <Path d="M5.4 13.2l.86 2.28 2.34.86-2.34.86-.86 2.4-.86-2.4-2.34-.86 2.34-.86.86-2.28z" />
        <Path d="M18.2 13.6l.62 1.66 1.68.62-1.68.62-.62 1.9-.62-1.9-1.68-.62 1.68-.62.62-1.66z" />
      </G>
    </Svg>
  );
}

function hslToHex(hue: number, saturation: number, lightness: number): string {
  const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation;
  const huePrime = hue * 6;
  const x = chroma * (1 - Math.abs((huePrime % 2) - 1));
  const match = lightness - chroma / 2;
  const [red, green, blue] = hslChannels(huePrime, chroma, x).map((channel) =>
    Math.round((channel + match) * 255),
  );

  return `#${toHex(red)}${toHex(green)}${toHex(blue)}`;
}

function hslChannels(huePrime: number, chroma: number, x: number): [number, number, number] {
  if (huePrime < 1) {
    return [chroma, x, 0];
  }
  if (huePrime < 2) {
    return [x, chroma, 0];
  }
  if (huePrime < 3) {
    return [0, chroma, x];
  }
  if (huePrime < 4) {
    return [0, x, chroma];
  }
  if (huePrime < 5) {
    return [x, 0, chroma];
  }
  return [chroma, 0, x];
}

function toHex(value: number): string {
  return value.toString(16).padStart(2, '0');
}
