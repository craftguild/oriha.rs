// Injected WebView JavaScript: PlantUML inline SVG runs, generic text measurement and request dispatch.
export const measurementDiagramSource = `      function measurePlantUmlInlineTextRequest(request) {
        var svg = createHiddenSvg();
        svg.setAttribute("height", "128");

        var text = document.createElementNS("http://www.w3.org/2000/svg", "text");
        text.setAttribute("x", "0");
        text.setAttribute("y", "64");

        var requestStyle = {};
        if (request.style) {
          Object.keys(request.style).forEach(function(property) {
            requestStyle[property] = request.style[property];
          });
        }
        if (request.fontFamily != null) {
          requestStyle.fontFamily = request.fontFamily;
        }
        if (request.fontSize != null) {
          requestStyle.fontSize = request.fontSize;
        }
        if (request.fontWeight != null) {
          requestStyle.fontWeight = request.fontWeight;
        }
        if (request.fontStyle != null) {
          requestStyle.fontStyle = request.fontStyle;
        }
        applySvgTextStyle(svg, requestStyle);
        applySvgTextStyle(text, requestStyle);

        request.runs.forEach(function(run, index) {
          var tspan = document.createElementNS("http://www.w3.org/2000/svg", "tspan");
          if (index === 0) {
            tspan.setAttribute("x", "0");
          }
          tspan.textContent = run.text || "";
          applySvgTextStyle(tspan, run.style);
          text.appendChild(tspan);
        });

        svg.appendChild(text);
        document.body.appendChild(svg);

        var textBox = svgBBox(text);
        var totalWidth = svgTextLength(text, textBox.width);
        var runResults = [];
        var fallbackX = 0;
        Array.prototype.forEach.call(text.children, function(tspan, index) {
          var run = request.runs[index];
          var box = svgBBox(tspan);
          var width = svgTextLength(tspan, box.width);
          var x = svgTextRunX(tspan, fallbackX);
          runResults.push({
            id: run.id,
            index: index,
            width: width,
            height: box.height,
            rects: [{
              x: x,
              y: box.y - textBox.y,
              width: width,
              height: box.height
            }]
          });
          fallbackX += width;
        });

        document.body.removeChild(svg);

        return {
          id: request.id,
          width: totalWidth,
          height: textBox.height,
          lineCount: 1,
          runs: runResults
        };
      }

      function measureRequest(request) {
        try {
          if (isPlantUmlInlineTextRequest(request)) {
            return sanitizeMeasurementResult(
              measurePlantUmlInlineTextRequest(request),
              request
            );
          }

          if (isMermaidRequest(request)) {
            return sanitizeMeasurementResult(measureMermaidRequest(request), request);
          }

          return sanitizeMeasurementResult(measureGenericTextRequest(request), request);
        } catch (_) {
          return fallbackMeasurementResult(request);
        }
      }

      function measureGenericTextRequest(request) {
        var element = document.createElement("div");
        element.style.position = "absolute";
        element.style.left = "0";
        element.style.top = "0";
        element.style.visibility = "hidden";
        element.style.whiteSpace = "pre-wrap";
        element.style.boxSizing = "border-box";
        element.style.overflowWrap = "break-word";
        var requestStyle = {};
        if (request.style) {
          Object.keys(request.style).forEach(function(property) {
            requestStyle[property] = request.style[property];
          });
        }
        if (request.fontFamily != null) {
          requestStyle.fontFamily = request.fontFamily;
        }
        if (request.fontSize != null) {
          requestStyle.fontSize = request.fontSize;
        }
        if (request.fontWeight != null) {
          requestStyle.fontWeight = request.fontWeight;
        }
        if (request.fontStyle != null) {
          requestStyle.fontStyle = request.fontStyle;
        }
        applyStyle(element, requestStyle);

        var width = request.width || requestStyle.width;
        var maxWidth = request.maxWidth || requestStyle.maxWidth;
        if (width != null) {
          element.style.width = cssValue("width", width);
        } else if (maxWidth != null) {
          element.style.maxWidth = cssValue("maxWidth", maxWidth);
        }

        var runResults = [];
        if (request.runs && request.runs.length > 0) {
          request.runs.forEach(function(run, index) {
            var span = document.createElement("span");
            span.textContent = run.text || "";
            applyStyle(span, run.style);
            element.appendChild(span);
          });
        } else {
          element.textContent = request.text || "";
        }

        document.body.appendChild(element);

        if (request.runs && request.runs.length > 0) {
          Array.prototype.forEach.call(element.children, function(span, index) {
            runResults.push(measureRun(span, request.runs[index], index));
          });
        }

        var bounds = element.getBoundingClientRect();
        var lineCount = Math.max(1, element.getClientRects().length);
        document.body.removeChild(element);

        var result = {
          id: request.id,
          width: bounds.width,
          height: bounds.height,
          lineCount: lineCount
        };
        if (runResults.length > 0) {
          result.runs = runResults;
        }
        return result;
      }

`;
