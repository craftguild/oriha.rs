//! Table, block-boundary and print layout overrides.
use super::*;

pub(super) fn full_table_data_css(enabled: bool) -> &'static str {
    if !enabled {
        return "";
    }
    r#"
.oriha-table-scroll-wrapper {
  overflow-x: visible;
  overflow-y: visible;
}

.oriha-table-scroll-wrapper .oriha-table {
  max-width: 100%;
  min-width: 0;
  table-layout: fixed;
  width: 100%;
}

.oriha-document table:not(.oriha-table) {
  max-width: 100%;
  table-layout: fixed;
  width: 100%;
}

.oriha-table th,
.oriha-table td,
.oriha-document table:not(.oriha-table) th,
.oriha-document table:not(.oriha-table) td {
  min-width: 0;
  overflow-wrap: anywhere;
  word-break: break-all;
  white-space: normal;
}"#
}

pub(super) fn block_borders_css(enabled: bool) -> &'static str {
    if !enabled {
        return "";
    }
    r#"
.oriha-code-block-wrapper,
.oriha-callout,
.oriha-image,
.oriha-plantuml,
.oriha-math-block,
.oriha-mermaid,
.oriha-diagram,
.oriha-foreign-html,
.oriha-foreign-html-wrapper {
  border: 1px solid var(--oriha-color-border);
}"#
}

pub(super) fn render_print_css(print: &PrintOptions) -> String {
    let page_size = sanitize_css_value(print.page_size.as_deref().unwrap_or("auto"), "auto");
    let margin = sanitize_css_value(print.margin.as_deref().unwrap_or("12mm"), "12mm");
    format!(
        r#"
@page {{
  size: {page_size};
  margin: {margin};
}}

@media print {{
  * {{
    -webkit-print-color-adjust: exact;
    print-color-adjust: exact;
  }}

  .oriha-document {{
    box-shadow: none;
    box-sizing: border-box;
    margin: 0;
    max-width: none;
    padding: 0 5px;
  }}

  .oriha-code-block,
  .oriha-callout,
  .oriha-blockquote,
  .oriha-document table {{
    break-inside: avoid;
    page-break-inside: avoid;
  }}

  .oriha-document table {{
    display: table;
    overflow: visible;
  }}

  .oriha-plantuml,
  .oriha-diagram,
  .oriha-math-block,
  .oriha-mermaid {{
    overflow: visible;
  }}

  .oriha-plantuml > svg,
  .oriha-diagram > svg,
  .oriha-math-block > svg,
  .oriha-mermaid > svg {{
    height: auto;
    max-width: 100%;
  }}

  .oriha-watermark {{
    bottom: 8mm;
    left: 50%;
    position: fixed;
    transform: translateX(-50%);
  }}

  .oriha-callout .oriha-code-block {{
    max-width: 100%;
    overflow: visible;
  }}

  .oriha-callout code {{
    white-space: pre-wrap;
  }}
}}
"#
    )
}

pub(super) fn table_border_css(style: TableBorderStyle) -> &'static str {
    match style {
        TableBorderStyle::All | TableBorderStyle::Inner => {
            r#".oriha-table th,
.oriha-table td,
.oriha-document table:not(.oriha-table) th,
.oriha-document table:not(.oriha-table) td {
  border-right: 1px solid var(--oriha-table-border);
  border-bottom: 1px solid var(--oriha-table-border);
}

.oriha-table th:last-child,
.oriha-table td:last-child,
.oriha-document table:not(.oriha-table) th:last-child,
.oriha-document table:not(.oriha-table) td:last-child {
  border-right: 0;
}

.oriha-table tr:last-child td,
.oriha-document table:not(.oriha-table) tr:last-child td {
  border-bottom: 0;
}"#
        }
        TableBorderStyle::Horizontal => {
            r#".oriha-table th,
.oriha-table td,
.oriha-document table:not(.oriha-table) th,
.oriha-document table:not(.oriha-table) td {
  border-bottom: 1px solid var(--oriha-table-border);
}

.oriha-table tr:last-child td,
.oriha-document table:not(.oriha-table) tr:last-child td {
  border-bottom: 0;
}"#
        }
        TableBorderStyle::Vertical => {
            r#".oriha-table th,
.oriha-table td,
.oriha-document table:not(.oriha-table) th,
.oriha-document table:not(.oriha-table) td {
  border-right: 1px solid var(--oriha-table-border);
}

.oriha-table th:last-child,
.oriha-table td:last-child,
.oriha-document table:not(.oriha-table) th:last-child,
.oriha-document table:not(.oriha-table) td:last-child {
  border-right: 0;
}"#
        }
        TableBorderStyle::None => {
            r#".oriha-table th,
.oriha-table td,
.oriha-document table:not(.oriha-table) th,
.oriha-document table:not(.oriha-table) td {
  border: 0;
}"#
        }
    }
}

pub(super) fn table_wrapper_border_css(style: TableBorderStyle) -> &'static str {
    match style {
        TableBorderStyle::All => "1px solid var(--oriha-table-border);",
        _ => "0;",
    }
}
