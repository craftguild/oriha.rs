import type { OrihaBuiltinLocale } from "../../generated/generatedBridgeTypes";

export type { OrihaBuiltinLocale } from "../../generated/generatedBridgeTypes";

export const DEFAULT_ORIHA_LOCALE =
  "en" satisfies OrihaBuiltinLocale;

// Expo runtime messages cover React Native UI around the renderer: toasts,
// debug sharing, external-link failures, and measurement errors. Core render
// chrome and settings option labels are localized by the Rust bridge.
export const ORIHA_MESSAGE_KEYS = [
  "codeCopy.success",
  "codeCopy.failure",
  "debug.shareRenderedHtml",
  "debug.shareDialogTitle",
  "debug.renderedHtmlSaved",
  "debug.shareFailed",
  "debug.fileSystemDirectoryUnavailable",
  "externalLink.openFailed",
  "measurement.cancelled",
  "measurement.timeout",
] as const;

export type OrihaLocale = OrihaBuiltinLocale | (string & {});
export type OrihaMessageKey =
  (typeof ORIHA_MESSAGE_KEYS)[number];
export type OrihaMessageValues = Readonly<
  Record<string, boolean | number | string | null | undefined>
>;
export type OrihaMessageTemplate =
  | string
  | ((values: OrihaMessageValues) => string);
export type OrihaMessages = Partial<
  Record<OrihaMessageKey, OrihaMessageTemplate>
>;
export type OrihaFormatMessage = (
  key: OrihaMessageKey,
  values?: OrihaMessageValues,
) => string;

export type OrihaI18nState = {
  locale: OrihaLocale;
  formatMessage: OrihaFormatMessage;
};

const EN_MESSAGES: Record<OrihaMessageKey, OrihaMessageTemplate> =
  {
    "codeCopy.success": "Copied",
    "codeCopy.failure": "Copy failed",
    "debug.shareRenderedHtml": "Share rendered HTML",
    "debug.shareDialogTitle": "Rendered HTML",
    "debug.renderedHtmlSaved": "Rendered HTML saved: {fileUri}",
    "debug.shareFailed": "HTML file share failed: {message}",
    "debug.fileSystemDirectoryUnavailable":
      "Expo FileSystem directory is not available",
    "externalLink.openFailed": "Could not open link",
    "measurement.cancelled": "Markdown measurement cancelled",
    "measurement.timeout": "Markdown measurement timed out after {timeoutMs}ms",
  };

const JA_MESSAGES: Record<OrihaMessageKey, OrihaMessageTemplate> =
  {
    "codeCopy.success": "コピーしました",
    "codeCopy.failure": "コピーに失敗しました",
    "debug.shareRenderedHtml": "レンダリング済みHTMLを共有",
    "debug.shareDialogTitle": "レンダリング済みHTML",
    "debug.renderedHtmlSaved":
      "レンダリング済みHTMLを保存しました: {fileUri}",
    "debug.shareFailed": "HTMLファイルの共有に失敗しました: {message}",
    "debug.fileSystemDirectoryUnavailable":
      "Expo FileSystem の保存先を利用できません",
    "externalLink.openFailed": "リンクを開けませんでした",
    "measurement.cancelled": "Markdown の計測をキャンセルしました",
    "measurement.timeout":
      "Markdown の計測が {timeoutMs}ms 以内に完了しませんでした",
  };

const KO_MESSAGES: Record<OrihaMessageKey, OrihaMessageTemplate> =
  {
    "codeCopy.success": "복사했습니다",
    "codeCopy.failure": "복사에 실패했습니다",
    "debug.shareRenderedHtml": "렌더링된 HTML 공유",
    "debug.shareDialogTitle": "렌더링된 HTML",
    "debug.renderedHtmlSaved": "렌더링된 HTML을 저장했습니다: {fileUri}",
    "debug.shareFailed": "HTML 파일 공유에 실패했습니다: {message}",
    "debug.fileSystemDirectoryUnavailable":
      "Expo FileSystem 저장 위치를 사용할 수 없습니다",
    "externalLink.openFailed": "링크를 열 수 없습니다",
    "measurement.cancelled": "Markdown 측정을 취소했습니다",
    "measurement.timeout":
      "Markdown 측정이 {timeoutMs}ms 안에 완료되지 않았습니다",
  };

const ZH_HANS_MESSAGES: Record<
  OrihaMessageKey,
  OrihaMessageTemplate
> = {
  "codeCopy.success": "已复制",
  "codeCopy.failure": "复制失败",
  "debug.shareRenderedHtml": "分享已渲染的 HTML",
  "debug.shareDialogTitle": "已渲染的 HTML",
  "debug.renderedHtmlSaved": "已保存已渲染的 HTML: {fileUri}",
  "debug.shareFailed": "HTML 文件分享失败: {message}",
  "debug.fileSystemDirectoryUnavailable": "无法使用 Expo FileSystem 目录",
  "externalLink.openFailed": "无法打开链接",
  "measurement.cancelled": "已取消 Markdown 测量",
  "measurement.timeout": "Markdown 测量在 {timeoutMs}ms 内未完成",
};

const ZH_HANT_MESSAGES: Record<
  OrihaMessageKey,
  OrihaMessageTemplate
> = {
  "codeCopy.success": "已複製",
  "codeCopy.failure": "複製失敗",
  "debug.shareRenderedHtml": "分享已轉譯的 HTML",
  "debug.shareDialogTitle": "已轉譯的 HTML",
  "debug.renderedHtmlSaved": "已儲存已轉譯的 HTML: {fileUri}",
  "debug.shareFailed": "HTML 檔案分享失敗: {message}",
  "debug.fileSystemDirectoryUnavailable": "無法使用 Expo FileSystem 目錄",
  "externalLink.openFailed": "無法開啟連結",
  "measurement.cancelled": "已取消 Markdown 測量",
  "measurement.timeout": "Markdown 測量未在 {timeoutMs}ms 內完成",
};

const ES_MESSAGES: Record<OrihaMessageKey, OrihaMessageTemplate> =
  {
    "codeCopy.success": "Copiado",
    "codeCopy.failure": "No se pudo copiar",
    "debug.shareRenderedHtml": "Compartir HTML renderizado",
    "debug.shareDialogTitle": "HTML renderizado",
    "debug.renderedHtmlSaved": "HTML renderizado guardado: {fileUri}",
    "debug.shareFailed":
      "No se pudo compartir el archivo HTML: {message}",
    "debug.fileSystemDirectoryUnavailable":
      "El directorio de Expo FileSystem no está disponible",
    "externalLink.openFailed": "No se pudo abrir el enlace",
    "measurement.cancelled": "Medición de Markdown cancelada",
    "measurement.timeout":
      "La medición de Markdown agotó el tiempo tras {timeoutMs} ms",
  };

const FR_MESSAGES: Record<OrihaMessageKey, OrihaMessageTemplate> =
  {
    "codeCopy.success": "Copié",
    "codeCopy.failure": "Échec de la copie",
    "debug.shareRenderedHtml": "Partager le HTML rendu",
    "debug.shareDialogTitle": "HTML rendu",
    "debug.renderedHtmlSaved": "HTML rendu enregistré : {fileUri}",
    "debug.shareFailed":
      "Échec du partage du fichier HTML : {message}",
    "debug.fileSystemDirectoryUnavailable":
      "Le répertoire Expo FileSystem n'est pas disponible",
    "externalLink.openFailed": "Impossible d'ouvrir le lien",
    "measurement.cancelled": "Mesure Markdown annulée",
    "measurement.timeout":
      "La mesure Markdown a expiré après {timeoutMs} ms",
  };

const BUILTIN_MESSAGES = {
  en: EN_MESSAGES,
  es: ES_MESSAGES,
  fr: FR_MESSAGES,
  ja: JA_MESSAGES,
  ko: KO_MESSAGES,
  "zh-Hans": ZH_HANS_MESSAGES,
  "zh-Hant": ZH_HANT_MESSAGES,
} satisfies Record<
  OrihaBuiltinLocale,
  Record<OrihaMessageKey, OrihaMessageTemplate>
>;

export function createOrihaI18n(
  locale: OrihaLocale = DEFAULT_ORIHA_LOCALE,
  messages?: OrihaMessages,
): OrihaI18nState {
  const builtinLocale = resolveBuiltinLocale(locale);
  const catalog = {
    ...EN_MESSAGES,
    ...BUILTIN_MESSAGES[builtinLocale],
    ...messages,
  };
  return {
    locale,
    formatMessage: (key, values) =>
      formatOrihaMessage(catalog, key, values),
  };
}

export function defaultOrihaFormatMessage(
  key: OrihaMessageKey,
  values?: OrihaMessageValues,
): string {
  return formatOrihaMessage(EN_MESSAGES, key, values);
}

export function formatOrihaMessage(
  messages: Readonly<Record<OrihaMessageKey, OrihaMessageTemplate>>,
  key: OrihaMessageKey,
  values: OrihaMessageValues = {},
): string {
  const template = messages[key] ?? EN_MESSAGES[key] ?? key;
  if (typeof template === "function") {
    return template(values);
  }
  return template.replace(/\{([A-Za-z0-9_]+)\}/g, (_match, name: string) => {
    const value = values[name];
    return value == null ? "" : String(value);
  });
}

function resolveBuiltinLocale(
  locale: OrihaLocale,
): OrihaBuiltinLocale {
  const normalized = locale.replace(/_/g, "-").toLowerCase();
  const parts = normalized.split("-");
  const language = parts[0];
  const regionOrScript = parts.slice(1).join("-");
  switch (language) {
    case "es":
      return "es";
    case "fr":
      return "fr";
    case "ja":
      return "ja";
    case "ko":
      return "ko";
    case "zh":
      if (
        regionOrScript.includes("hant") ||
        regionOrScript.includes("tw") ||
        regionOrScript.includes("hk") ||
        regionOrScript.includes("mo")
      ) {
        return "zh-Hant";
      }
      return "zh-Hans";
    case "en":
    default:
      return "en";
  }
}
