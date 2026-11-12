// Injected WebView JavaScript: WebView batch delivery and stable result-message protocol.
export function measurementBatchSource(MESSAGE_TYPE_BATCH_RESULT: string): string {
  return `      function measureBatch(payload) {
        try {
          var requests = Array.isArray(payload.requests) ? payload.requests : [];
          waitForFonts(requests).then(function() {
            var results = requests.map(measureRequest);
            window.ReactNativeWebView.postMessage(JSON.stringify({
              type: "${MESSAGE_TYPE_BATCH_RESULT}",
              id: payload.id,
              generationId: payload.generationId,
              results: results
            }));
          }, function(error) {
            window.ReactNativeWebView.postMessage(JSON.stringify({
              type: "${MESSAGE_TYPE_BATCH_RESULT}",
              id: payload.id,
              generationId: payload.generationId,
              results: [],
              error: error && error.message ? error.message : String(error)
            }));
          });
        } catch (error) {
          window.ReactNativeWebView.postMessage(JSON.stringify({
            type: "${MESSAGE_TYPE_BATCH_RESULT}",
            id: payload.id,
            generationId: payload.generationId,
            results: [],
            error: error && error.message ? error.message : String(error)
          }));
        }
      }

      window.__orihaMeasureTextBatch = function(payload) {
        measureBatch(payload);
      };
`;
}
