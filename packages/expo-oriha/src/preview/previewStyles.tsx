// Preview WebView container and debug-sharing overlay styles.
import { StyleSheet } from "react-native";

export const styles = StyleSheet.create({
  previewContainer: {
    flex: 1,
  },
  webView: {
    backgroundColor: "transparent",
    flex: 1,
  },
  debugOverlay: {
    alignItems: "flex-end",
    bottom: 18,
    gap: 8,
    left: 16,
    position: "absolute",
    right: 16,
  },
  debugButton: {
    alignItems: "center",
    backgroundColor: "#0f172a",
    borderColor: "#020617",
    borderRadius: 8,
    borderWidth: StyleSheet.hairlineWidth,
    justifyContent: "center",
    minHeight: 42,
    paddingHorizontal: 14,
  },
  debugButtonPressed: {
    opacity: 0.8,
  },
  debugButtonText: {
    color: "#ffffff",
    fontSize: 13,
    fontWeight: "700",
  },
});
