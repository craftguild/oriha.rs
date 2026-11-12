// Injected WebView JavaScript: CSS and SVG text style primitives shared by all measurement engines.
export const measurementStyleSource = `      var LENGTH_PROPS = {
        fontSize: true,
        lineHeight: true,
        letterSpacing: true,
        width: true,
        maxWidth: true,
        minWidth: true,
        height: true,
        maxHeight: true,
        minHeight: true,
        padding: true,
        paddingTop: true,
        paddingRight: true,
        paddingBottom: true,
        paddingLeft: true,
        margin: true,
        marginTop: true,
        marginRight: true,
        marginBottom: true,
        marginLeft: true
      };

      function cssValue(property, value) {
        if (value == null) {
          return "";
        }
        if (typeof value === "number" && LENGTH_PROPS[property]) {
          return value + "px";
        }
        return String(value);
      }

      function applyStyle(element, style) {
        if (!style) {
          return;
        }
        Object.keys(style).forEach(function(property) {
          var value = cssValue(property, style[property]);
          if (value) {
            element.style[property] = value;
          }
        });
      }

      function svgTextAttributeName(property) {
        switch (property) {
          case "fontFamily":
            return "font-family";
          case "fontSize":
            return "font-size";
          case "fontWeight":
            return "font-weight";
          case "fontStyle":
            return "font-style";
          case "letterSpacing":
            return "letter-spacing";
          case "textDecorationLine":
            return "text-decoration";
          default:
            return null;
        }
      }

      function applySvgTextStyle(element, style) {
        applyStyle(element, style);
        if (!style) {
          return;
        }
        Object.keys(style).forEach(function(property) {
          var attr = svgTextAttributeName(property);
          if (!attr) {
            return;
          }
          var value = cssValue(property, style[property]);
          if (value) {
            element.setAttribute(attr, value);
          }
        });
      }

      function mergedTextStyle(base, override) {
        var style = {};
        if (base) {
          Object.keys(base).forEach(function(property) {
            style[property] = base[property];
          });
        }
        if (override) {
          Object.keys(override).forEach(function(property) {
            style[property] = override[property];
          });
        }
        return style;
      }

`;
