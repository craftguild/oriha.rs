// Responsive preview panels and shared showcase control styles.
import { Platform, StyleSheet } from 'react-native';

export const PHONE_LAYOUT_MAX_WIDTH = 640;

export const styles = StyleSheet.create({
  screen: {
    backgroundColor: '#f8fafc',
    flex: 1,
  },
  appContent: {
    flex: 1,
  },
  settingsPanel: {
    borderBottomColor: '#dbe3ec',
    borderBottomWidth: StyleSheet.hairlineWidth,
    maxHeight: '48%',
  },
  controls: {
    gap: 12,
    padding: 12,
  },
  previewPanel: {
    flex: 1,
  },
  routerTabsPreview: {
    alignItems: 'stretch',
    backgroundColor: '#ffffff',
    borderTopColor: '#dbe3ec',
    borderTopWidth: StyleSheet.hairlineWidth,
    flexDirection: 'row',
    height: Platform.OS === 'ios' ? 74 : 64,
    paddingBottom: Platform.OS === 'ios' ? 10 : 0,
    paddingHorizontal: 12,
    paddingTop: 8,
  },
  routerTabsPreviewDark: {
    backgroundColor: '#0f172a',
    borderTopColor: '#263449',
  },
  routerTabItem: {
    alignItems: 'center',
    flex: 1,
    gap: 5,
    justifyContent: 'center',
  },
  routerTabIndicator: {
    backgroundColor: 'transparent',
    borderRadius: 2,
    height: 3,
    width: 26,
  },
  routerTabIndicatorSelected: {
    backgroundColor: '#2563eb',
  },
  routerTabLabel: {
    fontSize: 11,
    fontWeight: '700',
  },
  routerTabLabelLight: {
    color: '#64748b',
  },
  routerTabLabelDark: {
    color: '#cbd5e1',
  },
  routerTabLabelSelected: {
    color: '#2563eb',
  },
  optionRow: {
    gap: 6,
  },
  optionHeader: {
    gap: 2,
  },
  optionLabel: {
    color: '#475569',
    fontSize: 12,
    fontWeight: '700',
    textTransform: 'uppercase',
  },
  disabledReason: {
    color: '#8a4b18',
    fontSize: 12,
  },
  optionList: {
    flexDirection: 'row',
    gap: 8,
  },
  optionButton: {
    alignItems: 'center',
    backgroundColor: '#ffffff',
    borderColor: '#cbd5e1',
    borderRadius: 6,
    borderWidth: StyleSheet.hairlineWidth,
    justifyContent: 'center',
    minHeight: 34,
    paddingHorizontal: 12,
  },
  optionButtonSelected: {
    backgroundColor: '#0f172a',
    borderColor: '#0f172a',
  },
  optionButtonDisabled: {
    backgroundColor: '#f1f5f9',
    opacity: 0.64,
  },
  optionText: {
    color: '#334155',
    fontSize: 13,
    fontWeight: '600',
  },
  optionTextSelected: {
    color: '#ffffff',
  },
  optionTextDisabled: {
    color: '#64748b',
  },
  fontInstaller: {
    borderColor: '#dbe3ec',
    borderRadius: 8,
    borderWidth: StyleSheet.hairlineWidth,
    gap: 10,
    padding: 10,
  },
  usageAnalytics: {
    borderColor: '#dbe3ec',
    borderRadius: 8,
    borderWidth: StyleSheet.hairlineWidth,
    gap: 8,
    padding: 10,
  },
  usageAnalyticsHeader: {
    alignItems: 'center',
    flexDirection: 'row',
    gap: 12,
    justifyContent: 'space-between',
  },
  usageAnalyticsText: {
    flex: 1,
    gap: 2,
    minWidth: 0,
  },
  componentsSection: {
    borderColor: '#dbe3ec',
    borderRadius: 8,
    borderWidth: StyleSheet.hairlineWidth,
    gap: 10,
    padding: 10,
  },
  sectionTitle: {
    color: '#475569',
    fontSize: 12,
    fontWeight: '700',
    textTransform: 'uppercase',
  },
  bridgeVersionSection: {
    borderColor: '#dbe3ec',
    borderRadius: 8,
    borderWidth: StyleSheet.hairlineWidth,
    gap: 8,
    padding: 10,
  },
  bridgeVersionRows: {
    gap: 6,
  },
  metadataRow: {
    alignItems: 'center',
    flexDirection: 'row',
    gap: 12,
    justifyContent: 'space-between',
  },
  metadataLabel: {
    color: '#64748b',
    fontSize: 12,
    fontWeight: '700',
  },
  metadataValue: {
    color: '#1e293b',
    flexShrink: 1,
    fontSize: 12,
    fontVariant: ['tabular-nums'],
    fontWeight: '700',
    textAlign: 'right',
  },
  componentRow: {
    alignItems: 'center',
    backgroundColor: '#ffffff',
    borderColor: '#e2e8f0',
    borderRadius: 6,
    borderWidth: StyleSheet.hairlineWidth,
    flexDirection: 'row',
    gap: 10,
    justifyContent: 'space-between',
    paddingHorizontal: 10,
    paddingVertical: 8,
  },
  componentText: {
    flex: 1,
    minWidth: 0,
  },
  componentName: {
    color: '#1e293b',
    fontSize: 13,
    fontWeight: '700',
  },
  fontInstallerHeader: {
    alignItems: 'center',
    flexDirection: 'row',
    gap: 12,
    justifyContent: 'space-between',
  },
  fontInstallerTitle: {
    flex: 1,
    gap: 2,
    minWidth: 0,
  },
  fontList: {
    gap: 6,
  },
  fontListTitle: {
    color: '#334155',
    fontSize: 13,
    fontWeight: '700',
  },
  fontRow: {
    alignItems: 'center',
    backgroundColor: '#ffffff',
    borderColor: '#e2e8f0',
    borderRadius: 6,
    borderWidth: StyleSheet.hairlineWidth,
    flexDirection: 'row',
    gap: 10,
    justifyContent: 'space-between',
    paddingHorizontal: 10,
    paddingVertical: 8,
  },
  fontRowText: {
    flex: 1,
    minWidth: 0,
  },
  fontName: {
    color: '#1e293b',
    fontSize: 13,
    fontWeight: '700',
  },
  fontMeta: {
    color: '#64748b',
    fontSize: 12,
  },
  fontStatus: {
    color: '#166534',
    fontSize: 12,
  },
  fontError: {
    color: '#b42318',
    fontSize: 12,
  },
  smallButton: {
    alignItems: 'center',
    backgroundColor: '#0f172a',
    borderRadius: 6,
    justifyContent: 'center',
    minHeight: 32,
    minWidth: 82,
    paddingHorizontal: 10,
  },
  smallButtonDisabled: {
    opacity: 0.5,
  },
  smallButtonText: {
    color: '#ffffff',
    fontSize: 12,
    fontWeight: '700',
  },
  statusText: {
    color: '#64748b',
    fontSize: 13,
  },
  preview: {
    backgroundColor: 'transparent',
    flex: 1,
  },
  loading: {
    alignItems: 'center',
    flex: 1,
    gap: 8,
    justifyContent: 'center',
  },
  loadingText: {
    color: '#64748b',
  },
});
