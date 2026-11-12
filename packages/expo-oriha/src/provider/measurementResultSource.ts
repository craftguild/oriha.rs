// Injected WebView JavaScript: Measurement-result normalization, run geometry and SVG text primitives.
export const measurementResultSource = `      function fallbackMeasurementResult(request) {
        return {
          id: requestId(request),
          width: 0,
          height: 0,
          lineCount: 0
        };
      }

      function sanitizeRect(rect) {
        return {
          x: safeCoordinate(rect && rect.x),
          y: safeCoordinate(rect && rect.y),
          width: safeDimension(rect && rect.width),
          height: safeDimension(rect && rect.height)
        };
      }

      function sanitizeMeasurementResult(result, request) {
        if (!result) {
          return fallbackMeasurementResult(request);
        }
        var sanitized = {
          id: result.id != null ? String(result.id) : requestId(request),
          width: safeDimension(result.width),
          height: safeDimension(result.height),
          lineCount: Math.max(0, Math.round(finiteNumber(result.lineCount, 0)))
        };
        if (result.left != null) {
          sanitized.left = safeCoordinate(result.left);
        }
        if (result.right != null) {
          sanitized.right = safeCoordinate(result.right);
        }
        if (result.rawWidth != null) {
          sanitized.rawWidth = safeDimension(result.rawWidth);
        }
        if (Array.isArray(result.runs)) {
          sanitized.runs = result.runs.map(function(run, index) {
            var rects = Array.isArray(run && run.rects) ? run.rects : [];
            return {
              id: run && run.id != null ? String(run.id) : undefined,
              index: Math.max(0, Math.round(finiteNumber(run && run.index, index))),
              width: safeDimension(run && run.width),
              height: safeDimension(run && run.height),
              rects: rects.map(sanitizeRect)
            };
          });
        }
        return sanitized;
      }

      function rectToJson(rect) {
        return {
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height
        };
      }

      function measureRun(span, run, index) {
        var rects = Array.prototype.map.call(span.getClientRects(), rectToJson);
        var bounds = span.getBoundingClientRect();
        return {
          id: run.id,
          index: index,
          width: bounds.width,
          height: bounds.height,
          rects: rects
        };
      }

      function isPlantUmlInlineTextRequest(request) {
        return request &&
          request.engine === "plantuml" &&
          request.kind === "inlineText" &&
          request.runs &&
          request.runs.length > 0;
      }

      function isMermaidRequest(request) {
        return request && request.engine === "mermaid";
      }

      function svgBBox(element) {
        try {
          return element.getBBox();
        } catch (_) {
          return { x: 0, y: 0, width: 0, height: 0 };
        }
      }

      function svgTextLength(element, fallbackWidth) {
        try {
          var length = element.getComputedTextLength();
          if (Number.isFinite(length)) {
            return length;
          }
        } catch (_) {}
        return fallbackWidth;
      }

      function svgTextRunX(element, fallbackX) {
        try {
          var text = element.textContent || "";
          if (text.length > 0) {
            var point = element.getStartPositionOfChar(0);
            if (Number.isFinite(point.x)) {
              return point.x;
            }
          }
        } catch (_) {}
        return fallbackX;
      }

      function createHiddenSvg() {
        var svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        svg.style.position = "absolute";
        svg.style.left = "0";
        svg.style.top = "0";
        svg.style.visibility = "hidden";
        svg.style.overflow = "visible";
        svg.setAttribute("width", "4096");
        svg.setAttribute("height", "256");
        return svg;
      }

      function measureMermaidSvgTextRequest(request) {
        var svg = createHiddenSvg();
        var text = document.createElementNS("http://www.w3.org/2000/svg", "text");
        text.setAttribute("x", "0");
        text.setAttribute("y", "128");
        text.textContent = requestText(request);

        var requestStyle = copyRequestTextStyle(request);
        applySvgTextStyle(svg, requestStyle);
        applySvgTextStyle(text, requestStyle);

        svg.appendChild(text);
        document.body.appendChild(svg);

        var box = svgBBox(text);
        var computedLength = svgTextLength(text, box.width);
        document.body.removeChild(svg);

        var left = Math.max(0, -safeCoordinate(box.x));
        var width = safeDimension(box.width);
        var result = {
          id: requestId(request),
          width: width,
          height: safeDimension(box.height),
          lineCount: 1,
          left: left,
          right: Math.max(0, safeCoordinate(box.x) + width)
        };

        if (request.kind === "svgComputedLength") {
          result.width = safeDimension(computedLength);
          result.rawWidth = result.width;
          result.left = 0;
          result.right = result.width;
        } else {
          result.rawWidth = safeDimension(computedLength);
        }

        return result;
      }

`;
