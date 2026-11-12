// Injected WebView JavaScript: Numeric defaults, request styles, font loading and wrap-mode decisions.
export const measurementFontSource = `      function finiteNumber(value, fallback) {
        var number = typeof value === "number" ? value : Number(value);
        return Number.isFinite(number) ? number : fallback;
      }

      function safeDimension(value) {
        return Math.max(0, finiteNumber(value, 0));
      }

      function safeCoordinate(value) {
        return finiteNumber(value, 0);
      }

      function requestId(request) {
        if (request && request.id != null) {
          return String(request.id);
        }
        return "";
      }

      function requestText(request) {
        if (request && request.text != null) {
          return String(request.text);
        }
        return "";
      }

      function copyRequestTextStyle(request) {
        var requestStyle = {};
        if (request && request.style) {
          Object.keys(request.style).forEach(function(property) {
            requestStyle[property] = request.style[property];
          });
        }
        if (request && request.fontFamily != null) {
          requestStyle.fontFamily = request.fontFamily;
        }
        if (request && request.fontSize != null) {
          requestStyle.fontSize = request.fontSize;
        }
        if (request && request.fontWeight != null) {
          requestStyle.fontWeight = request.fontWeight;
        }
        if (request && request.fontStyle != null) {
          requestStyle.fontStyle = request.fontStyle;
        }
        return requestStyle;
      }

      function fontLoadStyle(style) {
        if (!style || style.fontFamily == null) {
          return null;
        }
        var fontSize = safeDimension(firstDefined(style.fontSize, 16));
        if (fontSize <= 0) {
          fontSize = 16;
        }
        var fontStyle = style.fontStyle != null ? String(style.fontStyle) : "normal";
        var fontWeight = style.fontWeight != null ? String(style.fontWeight) : "normal";
        return fontStyle + " " + fontWeight + " " + fontSize + "px " + String(style.fontFamily);
      }

      function collectFontLoadRequests(request, output) {
        var requestStyle = copyRequestTextStyle(request);
        var requestFont = fontLoadStyle(requestStyle);
        if (requestFont) {
          output.push({
            font: requestFont,
            text: requestText(request)
          });
        }
        if (request && Array.isArray(request.runs)) {
          request.runs.forEach(function(run) {
            var runStyle = mergedTextStyle(requestStyle, run && run.style);
            var runFont = fontLoadStyle(runStyle);
            if (runFont) {
              output.push({
                font: runFont,
                text: run && run.text != null ? String(run.text) : ""
              });
            }
          });
        }
      }

      function waitForFonts(requests) {
        if (!document.fonts) {
          return Promise.resolve();
        }
        var loads = [];
        var seen = {};
        var fontRequests = [];
        requests.forEach(function(request) {
          collectFontLoadRequests(request, fontRequests);
        });
        fontRequests.forEach(function(fontRequest) {
          var key = fontRequest.font + "\\n" + fontRequest.text;
          if (seen[key]) {
            return;
          }
          seen[key] = true;
          if (document.fonts.load) {
            loads.push(
              document.fonts.load(fontRequest.font, fontRequest.text || "A")
                .catch(function() {
                  return [];
                })
            );
          }
        });
        if (document.fonts.ready) {
          loads.push(
            document.fonts.ready.catch(function() {
              return document.fonts;
            })
          );
        }
        return Promise.all(loads).then(function() {
          return undefined;
        });
      }

      function firstDefined(primary, secondary) {
        return primary != null ? primary : secondary;
      }

      function normalizedWrapMode(request, fallback) {
        var value = request && request.wrapMode != null
          ? request.wrapMode
          : fallback;
        return String(value || "")
          .trim()
          .toLowerCase()
          .replace(/_/g, "-")
          .replace(/\\s+/g, "-");
      }

      function applyWrapMode(element, request, fallback) {
        var mode = normalizedWrapMode(request, fallback);
        switch (mode) {
          case "none":
          case "no-wrap":
          case "nowrap":
            element.style.whiteSpace = "pre";
            element.style.overflowWrap = "normal";
            element.style.wordBreak = "normal";
            return;
          case "normal":
            element.style.whiteSpace = "normal";
            element.style.overflowWrap = "normal";
            element.style.wordBreak = "normal";
            return;
          case "pre":
            element.style.whiteSpace = "pre";
            element.style.overflowWrap = "normal";
            element.style.wordBreak = "normal";
            return;
          case "pre-line":
            element.style.whiteSpace = "pre-line";
            element.style.overflowWrap = "break-word";
            element.style.wordBreak = "normal";
            return;
          case "anywhere":
            element.style.whiteSpace = "pre-wrap";
            element.style.overflowWrap = "anywhere";
            element.style.wordBreak = "break-word";
            return;
          case "break-word":
          case "breakword":
            element.style.whiteSpace = "pre-wrap";
            element.style.overflowWrap = "break-word";
            element.style.wordBreak = "normal";
            return;
          case "pre-wrap":
          case "preserve":
          default:
            element.style.whiteSpace = "pre-wrap";
            element.style.overflowWrap = "break-word";
            element.style.wordBreak = "normal";
            return;
        }
      }

`;
