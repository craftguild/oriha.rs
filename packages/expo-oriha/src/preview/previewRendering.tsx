// Bridge layout and patch rendering with typed measurement callbacks.
import { useOrihaContext } from "../provider/OrihaProvider";
import {
  prepareMarkdownLayout,
  prepareMarkdownPatch,
  renderPreparedMarkdown,
  renderPreparedMarkdownPatch,
} from "../bridge/render";
import type { BridgeRenderInput, RenderOutput, RenderPatchOutput, RenderSnapshot } from "../types";

export type RenderPreviewOptions = {
  input: BridgeRenderInput;
  generationId: number;
  measureTextBatch?: OrihaMeasureTextBatch;
};

export type RenderPreviewPatchOptions = RenderPreviewOptions & {
  previousSnapshot?: RenderSnapshot | null;
};

export type PendingPreviewPatch = {
  generationId: number;
  input: BridgeRenderInput;
  inputKey: string;
  patchOutput: RenderPatchOutput;
  timeoutId?: ReturnType<typeof setTimeout>;
};

export type OrihaMeasureTextBatch = NonNullable<
  ReturnType<typeof useOrihaContext>
>["measureTextBatch"];

export async function renderPreview({
  input,
  generationId,
  measureTextBatch,
}: RenderPreviewOptions): Promise<RenderOutput> {
  const layout = await prepareMarkdownLayout(input);
  const measureRequests = layout.textMeasurements.requests;
  const measureResults =
    measureTextBatch && measureRequests.length > 0
      ? await measureTextBatch(measureRequests, { generationId })
      : [];
  return renderPreparedMarkdown(layout, measureResults);
}

export async function renderPreviewPatch({
  input,
  generationId,
  measureTextBatch,
  previousSnapshot,
}: RenderPreviewPatchOptions): Promise<RenderPatchOutput> {
  const preparedPatch = await prepareMarkdownPatch(previousSnapshot, input);
  const measureRequests = preparedPatch.textMeasurements.requests;
  const measureResults =
    measureTextBatch && measureRequests.length > 0
      ? await measureTextBatch(measureRequests, { generationId })
      : [];
  return renderPreparedMarkdownPatch(preparedPatch, measureResults);
}
