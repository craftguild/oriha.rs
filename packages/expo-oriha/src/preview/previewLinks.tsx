// External links and code-copy notifications with localized toast feedback.
import { Linking } from "react-native";
import { OrihaToastPosition, OrihaToastVariant } from "../provider/OrihaEnums";
import { defaultOrihaFormatMessage } from "../runtime/expoRuntimeI18n";
import { CODE_COPY_MESSAGE_TYPE, EXTERNAL_LINK_MESSAGE_TYPE, shouldOpenExternally } from "./orihaPreviewHtml";
import type { OrihaToastOptions } from "../provider/OrihaProvider";
import type { OrihaFormatMessage } from "../runtime/expoRuntimeI18n";

export type CodeCopyMessage = {
  type?: string;
  status?: "success" | "error";
  message?: string;
};

export type ExternalLinkMessage = {
  type?: string;
  url?: string;
};

export function handleExternalLinkMessage(
  data: string,
  showToast?: (
    message: string,
    options?: OrihaToastOptions,
  ) => () => void,
  formatMessage: OrihaFormatMessage = defaultOrihaFormatMessage,
) {
  let message: ExternalLinkMessage;
  try {
    message = JSON.parse(data) as ExternalLinkMessage;
  } catch {
    return;
  }

  if (message.type !== EXTERNAL_LINK_MESSAGE_TYPE || !message.url) {
    return;
  }

  if (!shouldOpenExternally(message.url)) {
    return;
  }

  openExternalUrl(message.url, showToast, formatMessage);
}

export function openExternalUrl(
  url: string,
  showToast?: (
    message: string,
    options?: OrihaToastOptions,
  ) => () => void,
  formatMessage: OrihaFormatMessage = defaultOrihaFormatMessage,
) {
  Linking.openURL(url).catch(() => {
    showToast?.(formatMessage("externalLink.openFailed"), {
      position: OrihaToastPosition.Top,
      variant: OrihaToastVariant.Err,
    });
  });
}

export function handleCodeCopyMessage(
  data: string,
  showToast?: (
    message: string,
    options?: OrihaToastOptions,
  ) => () => void,
  formatMessage: OrihaFormatMessage = defaultOrihaFormatMessage,
) {
  if (!showToast) {
    return;
  }

  let message: CodeCopyMessage;
  try {
    message = JSON.parse(data) as CodeCopyMessage;
  } catch {
    return;
  }

  if (message.type !== CODE_COPY_MESSAGE_TYPE) {
    return;
  }

  const isError = message.status === "error";
  const toastMessage =
    message.message ??
    formatMessage(isError ? "codeCopy.failure" : "codeCopy.success");
  showToast(toastMessage, {
    position: OrihaToastPosition.Top,
    variant: isError
      ? OrihaToastVariant.Err
      : OrihaToastVariant.Success,
  });
}
