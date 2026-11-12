// Injected WebView JavaScript: HTML text sizing, wrapping and Mermaid measurement dispatch.
export const measurementHtmlSource = `      function appendTextContent(element, request) {
        if (request && Array.isArray(request.runs) && request.runs.length > 0) {
          request.runs.forEach(function(run) {
            var span = document.createElement("span");
            span.textContent = run && run.text != null ? String(run.text) : "";
            applyStyle(span, run && run.style);
            element.appendChild(span);
          });
          return;
        }

        var span = document.createElement("span");
        span.textContent = requestText(request);
        element.appendChild(span);
      }

      function contentLineCount(element) {
        var count = 0;
        Array.prototype.forEach.call(element.children, function(child) {
          count += child.getClientRects().length;
        });
        if (count === 0) {
          count = element.getClientRects().length;
        }
        return Math.max(1, count);
      }

      function createHtmlTextMeasureElement(request, options) {
        var element = document.createElement("div");
        var requestStyle = copyRequestTextStyle(request);
        element.style.position = "absolute";
        element.style.left = "0";
        element.style.top = "0";
        element.style.visibility = "hidden";
        element.style.boxSizing = "border-box";
        element.style.display = "inline-block";
        element.style.pointerEvents = "none";
        applyStyle(element, requestStyle);
        applyWrapMode(element, request, options.wrapMode);

        if (options.constrain) {
          var width = firstDefined(request && request.width, requestStyle.width);
          var maxWidth = firstDefined(request && request.maxWidth, requestStyle.maxWidth);
          if (width != null) {
            element.style.width = cssValue("width", width);
          } else if (maxWidth != null) {
            element.style.maxWidth = cssValue("maxWidth", maxWidth);
          }
        }

        appendTextContent(element, request);
        return element;
      }

      function measureHtmlText(request, options) {
        var element = createHtmlTextMeasureElement(request, options);
        document.body.appendChild(element);
        var bounds = element.getBoundingClientRect();
        var result = {
          id: requestId(request),
          width: bounds.width,
          height: bounds.height,
          lineCount: contentLineCount(element)
        };
        document.body.removeChild(element);
        return result;
      }

      function measureHtmlRawWidth(request) {
        var element = createHtmlTextMeasureElement(request, {
          constrain: false,
          wrapMode: "none"
        });
        document.body.appendChild(element);
        var width = element.getBoundingClientRect().width;
        document.body.removeChild(element);
        return width;
      }

      function measureMermaidHtmlTextRequest(request) {
        var kind = request && request.kind;
        var isWrapped = kind !== "text";
        var result = measureHtmlText(request, {
          constrain: isWrapped,
          wrapMode: isWrapped ? "pre-wrap" : "none"
        });
        if (kind === "wrappedRaw" || kind === "wrappedWithRawWidth") {
          result.rawWidth = measureHtmlRawWidth(request);
        }
        return result;
      }

      function measureMermaidRequest(request) {
        switch (request && request.kind) {
          case "text":
          case "wrapped":
          case "wrappedRaw":
          case "wrappedWithRawWidth":
            return measureMermaidHtmlTextRequest(request);
          case "svgComputedLength":
          case "svgBBoxX":
          case "svgBBoxXAsciiOverhang":
          case "svgTitleBBoxX":
          case "svgSimpleBBoxWidth":
          case "svgSimpleBBoxHeight":
            return measureMermaidSvgTextRequest(request);
          default:
            return fallbackMeasurementResult(request);
        }
      }

`;
