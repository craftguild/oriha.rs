// Provider host, overlay, toast and hidden measurement WebView styles.
import { StyleSheet } from "react-native";

export const styles = StyleSheet.create({
  root: {
    flex: 1,
  },
  overlayHost: {
    ...StyleSheet.absoluteFillObject,
    zIndex: 10_000,
    elevation: 10_000,
  },
  overlaySlot: {
    ...StyleSheet.absoluteFillObject,
  },
  overlayTop: {
    alignItems: "stretch",
    justifyContent: "flex-start",
  },
  overlayCenter: {
    alignItems: "center",
    justifyContent: "center",
  },
  overlayBottom: {
    alignItems: "stretch",
    justifyContent: "flex-end",
  },
  overlayFill: {
    alignItems: "stretch",
    justifyContent: "flex-start",
  },
  toastHost: {
    ...StyleSheet.absoluteFillObject,
    zIndex: 10_001,
    elevation: 10_001,
  },
  toastSlot: {
    ...StyleSheet.absoluteFillObject,
    alignItems: "center",
    paddingHorizontal: 16,
  },
  toastSlotTop: {
    justifyContent: "flex-start",
    paddingTop: 48,
  },
  toastSlotCenter: {
    justifyContent: "center",
  },
  toastSlotBottom: {
    justifyContent: "flex-end",
    paddingBottom: 24,
  },
  toastItem: {
    marginVertical: 4,
  },
  toast: {
    alignItems: "center",
    alignSelf: "center",
    borderRadius: 999,
    flexDirection: "row",
    maxWidth: "100%",
    paddingHorizontal: 16,
    paddingVertical: 10,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 6 },
    shadowOpacity: 0.18,
    shadowRadius: 14,
  },
  toastIcon: {
    marginRight: 8,
  },
  toastText: {
    fontSize: 14,
    fontWeight: "600",
    lineHeight: 18,
  },
  measurementHost: {
    position: "absolute",
    left: -10_000,
    top: 0,
    width: 1_024,
    height: 1_024,
    opacity: 0,
    overflow: "hidden",
  },
  measurementWebView: {
    width: 1_024,
    height: 1_024,
    backgroundColor: "transparent",
  },
});
