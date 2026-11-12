use super::*;
use crate::types::{AppearanceOptions, FontFamilySpec, RenderOptions, WatermarkOptions};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[test]
fn renders_heading_and_code_block() {
    let output = render(RenderInput {
        markdown: "## Heading\n\n```rust\nlet x = 1;\n```".to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains("oriha-heading"));
    assert!(output.html.contains("oriha-code-block-wrapper"));
    assert!(output.html.contains("oriha-code-block-header"));
    assert!(output.html.contains("oriha-code-block-language"));
    assert!(output.html.contains("oriha-code-copy-button"));
    assert!(output.html.contains("oriha-code-copy-icon"));
    assert!(output.html.contains("oriha-code-copy-label"));
    assert!(output.html.contains("oriha-code-gutter"));
    assert!(output.html.contains("oriha-code-line-number"));
    assert!(output.html.contains("oriha-code-scroll"));
    assert!(output.html.contains("oriha-code-lines"));
    assert!(output.html.contains("oriha-code-line"));
    assert!(output.html.contains("token keyword"));
    assert!(output
        .source_map
        .iter()
        .any(|block| block.kind == "heading"));
    assert!(output
        .source_map
        .iter()
        .any(|block| block.kind == "codeBlock"));
}

#[test]
fn heading_ids_are_slugged_for_page_links() {
    let output = render(RenderInput {
        markdown: concat!(
            "## Contents\n\n",
            "- [Inline markup](#inline-markup)\n",
            "- [Duplicate](#inline-markup-2)\n\n",
            "## Inline Markup\n\n",
            "## Inline Markup\n",
        )
        .to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains(r##"href="#inline-markup""##));
    assert!(output.html.contains(r##"href="#inline-markup-2""##));
    assert!(output.html.contains(
        r#"<h2 id="inline-markup" class="oriha-heading oriha-heading--h2" data-oriha-block-kind="heading""#
    ));
    assert!(output.html.contains(
        r#"<h2 id="inline-markup-2" class="oriha-heading oriha-heading--h2" data-oriha-block-kind="heading""#
    ));
}

#[test]
fn gfm_footnotes_render_references_and_definition_section() {
    let output = render(RenderInput {
        markdown: concat!(
            "A paragraph with a footnote.[^note]\n",
            "The same footnote can be referenced again.[^note]\n\n",
            "[^note]: Footnote **body** with [link](#target).\n",
        )
        .to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains(
        r##"<sup id="fnref-note" class="oriha-footnote-ref"><a href="#fn-note" class="oriha-footnote-ref__link" data-oriha-footnote-ref="true">1</a></sup>"##
    ));
    assert!(output.html.contains(
        r##"<sup id="fnref-note-2" class="oriha-footnote-ref"><a href="#fn-note" class="oriha-footnote-ref__link" data-oriha-footnote-ref="true">1</a></sup>"##
    ));
    assert!(output.html.contains(r#"class="oriha-footnotes""#));
    assert!(output
        .html
        .contains(r#"<li id="fn-note" class="oriha-footnotes__item" value="1">"#));
    assert!(output.html.contains(r#"<strong>body</strong>"#));
    assert!(output.html.contains(r##"href="#target""##));
    assert!(!output.html.contains("[^note]:"));
}

#[test]
fn locale_ja_localizes_rendered_chrome() {
    let output = render(RenderInput {
        markdown: concat!(
            "```rust\n",
            "let value = 1;\n",
            "```\n\n",
            "脚注の参照です。[^note]\n\n",
            "[^note]: 脚注本文です。\n",
        )
        .to_string(),
        locale: Some("ja-JP".to_string()),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains(r#"aria-label="コードをコピー""#));
    assert!(output.html.contains(
        r#"<span class="oriha-code-copy-label" data-oriha-copy-label="true">コピー</span>"#
    ));
    assert!(output
        .html
        .contains(r#"<h2 id="oriha-footnotes-label" class="oriha-footnotes__title">脚注</h2>"#));
    assert!(output.html.contains(r#"aria-label="参照元に戻る""#));
}

#[test]
fn syntax_highlight_flag_renders_plain_escaped_code_lines() {
    let output = render(RenderInput {
        markdown: "```rust\nlet x = <tag>;\n```\n".to_string(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            syntax_highlight: false,
            ..FeatureFlags::default()
        },
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains("oriha-code-block-header"));
    assert!(output.html.contains("oriha-code-gutter"));
    assert!(output.html.contains("oriha-code-scroll"));
    assert!(output.html.contains("oriha-code-line"));
    assert!(output.html.contains("let x = &lt;tag&gt;;"));
    assert!(!output.html.contains(r#"class="token "#));
}

#[test]
fn standalone_images_render_with_containers_and_optional_captions() {
    let output = render(RenderInput {
        markdown: concat!(
            "![Plain image](https://example.com/plain.jpg)\n\n",
            "![Wide dunes](https://images.example/dune.jpg?width=1600&height=900 \"Wide caption\")\n\n",
            "Text before ![Inline image](https://example.com/inline.jpg?w=120&h=80 \"Inline title\") text after.\n",
        )
        .to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains(r#"class="oriha-image""#));
    assert!(output.html.contains(r#"data-oriha-block-kind="image""#));
    assert!(output.html.contains(r#"style="width: 300px;""#));
    assert!(output.html.contains(
        r#"<img class="oriha-image__img" src="https://example.com/plain.jpg" alt="Plain image" style="width: 100%; height: auto;" />"#
    ));
    assert!(output
        .html
        .contains(r#"class="oriha-image oriha-image--captioned""#));
    assert!(output.html.contains(r#"data-oriha-image-caption="true""#));
    assert!(output.html.contains(
        r#"<img class="oriha-image__img" src="https://images.example/dune.jpg?width=1600&amp;height=900" alt="Wide dunes" width="1600" height="900" style="width: 100%; height: auto;" />"#
    ));
    assert!(output
        .html
        .contains(r#"<figcaption class="oriha-image__caption">Wide caption</figcaption>"#));
    assert!(output.html.contains(
        r#"<img class="oriha-inline-image" src="https://example.com/inline.jpg?w=120&amp;h=80" alt="Inline image" title="Inline title" width="120" height="80" style="width: 120px; height: auto;" />"#
    ));
    assert!(!output
        .html
        .contains(r#"<figcaption class="oriha-image__caption">Inline title</figcaption>"#));
    assert!(output.source_map.iter().any(|block| block.kind == "image"));
}

#[test]
fn code_language_label_uses_canonical_display_names() {
    assert_eq!(code_language_label("plantuml"), "PlantUML");
    assert_eq!(code_language_label("puml"), "PlantUML");
    assert_eq!(code_language_label("js"), "JavaScript");
    assert_eq!(code_language_label("typescript"), "TypeScript");
    assert_eq!(code_language_label("postgresql"), "PostgreSQL");
    assert_eq!(code_language_label(""), "Plain text");
    assert_eq!(code_language_label("custom-dsl"), "custom-dsl");
}

#[test]
fn parsed_documents_reuse_parser_and_isolate_render_hook_mutations() {
    struct Lifecycle(Arc<Mutex<Vec<&'static str>>>);
    impl RenderHook for Lifecycle {
        fn before_document_parse(
            &self,
            input: &mut RenderInput,
            state: &mut RenderState,
            _context: &mut RenderContext,
        ) {
            self.0.lock().unwrap().push("before_parse");
            input.mode = RenderMode::SemanticHtml;
            state.attrs.insert("parse_state".into(), "retained".into());
            state.diagnostics.push(RenderDiagnostic {
                severity: Severity::Warning,
                code: "test.parse_diagnostic".into(),
                message: "retained parse warning".into(),
                source_range: None,
                block_id: None,
                engine: DiagnosticEngine::Core,
            });
        }
        fn after_document_parse(
            &self,
            document: &mut Document,
            state: &mut RenderState,
            context: &mut RenderContext,
        ) {
            self.0.lock().unwrap().push("after_parse");
            assert_eq!(state.attrs.get("parse_state").unwrap(), "retained");
            context.mode = RenderMode::SemanticHtml;
            if let BlockKind::Paragraph(p) = &mut document.blocks[1].kind {
                if let InlineKind::Text(text) = &mut p.children[0].kind {
                    text.push('!');
                }
            }
        }
        fn before_output_finalize(
            &self,
            _document: &Document,
            state: &mut RenderState,
            _context: &mut RenderContext,
        ) {
            self.0.lock().unwrap().push("before_finalize");
            assert_eq!(state.attrs.get("parse_state").unwrap(), "retained");
            assert!(!state.attrs.contains_key("render_only"));
            state.attrs.insert("render_only".into(), "one".into());
        }
        fn before_block_render(
            &self,
            block: &mut BlockNode,
            _fragment: &mut RenderFragment,
            _state: &mut RenderState,
            _context: &RenderContext,
        ) {
            self.0.lock().unwrap().push("block_render");
            if let BlockKind::Paragraph(p) = &mut block.kind {
                p.children
                    .push(InlineKind::Text(" working copy".into()).into());
            }
        }
        fn after_output_finalize(
            &self,
            _document: &Document,
            _output: &mut RenderOutput,
            _state: &mut RenderState,
            _context: &mut RenderContext,
        ) {
            self.0.lock().unwrap().push("after_finalize");
        }
    }
    let events = Arc::new(Mutex::new(Vec::new()));
    let engine = OrihaEngine::default().add_hook(Lifecycle(Arc::clone(&events)));
    crate::parser::PARSE_CALLS.with(|calls| calls.set((0, 0)));
    let parsed = engine.parse(RenderInput {
        markdown: "# @alice\n\n@bob".into(),
        ..RenderInput::default_for_test()
    });
    let parse_calls = crate::parser::PARSE_CALLS.with(|calls| calls.get());
    assert!(parse_calls.0 > 0 && parse_calls.1 > 0);
    assert_eq!(*events.lock().unwrap(), ["before_parse", "after_parse"]);
    assert!(matches!(parsed.context.mode, RenderMode::SemanticHtml));
    assert!(matches!(parsed.input.mode, RenderMode::SemanticHtml));
    assert_eq!(parsed.diagnostics()[0].code, "test.parse_diagnostic");
    let regions: Vec<_> = parsed.text_regions().collect();
    assert_eq!(regions[1].text, "@bob!");
    assert!(regions[1]
        .mappings
        .iter()
        .all(|m| m.mapping.provenance == crate::TextProvenance::Generated
            && m.mapping.source.is_empty()));
    let first = engine.render_parsed(&parsed);
    let second = engine.render_parsed(&parsed);
    assert_eq!(
        crate::parser::PARSE_CALLS.with(|calls| calls.get()),
        parse_calls
    );
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
    assert!(first.html.contains("@bob! working copy"));
    assert!(first
        .diagnostics
        .iter()
        .any(|d| d.code == "test.parse_diagnostic"));
    assert_eq!(parsed.text_regions().nth(1).unwrap().text, "@bob!");
    let trace = events.lock().unwrap();
    assert_eq!(trace.iter().filter(|e| **e == "before_parse").count(), 1);
    assert_eq!(trace.iter().filter(|e| **e == "after_parse").count(), 1);
    assert_eq!(
        &trace[2..6],
        [
            "before_finalize",
            "block_render",
            "block_render",
            "after_finalize"
        ]
    );
    assert_eq!(&trace[6..10], &trace[2..6]);
}

#[test]
fn parse_boundary_contains_panics_and_marks_replaced_input_generated() {
    struct Replace;
    impl RenderHook for Replace {
        fn before_document_parse(
            &self,
            input: &mut RenderInput,
            _state: &mut RenderState,
            _context: &mut RenderContext,
        ) {
            input.markdown = "replacement @alice".into();
        }
    }
    let parsed = OrihaEngine::default().add_hook(Replace).parse(RenderInput {
        markdown: "original".into(),
        ..RenderInput::default_for_test()
    });
    assert_eq!(parsed.source(), "original");
    assert!(parsed.text_regions().all(|r| r
        .mappings
        .iter()
        .all(|m| m.mapping.provenance == crate::TextProvenance::Generated
            && m.mapping.source.is_empty())));
    struct Panic;
    impl RenderHook for Panic {
        fn before_document_parse(
            &self,
            _: &mut RenderInput,
            _: &mut RenderState,
            _: &mut RenderContext,
        ) {
            panic!("parse failure");
        }
    }
    let doc = OrihaEngine::default()
        .add_hook(Panic)
        .parse(RenderInput::default_for_test());
    assert_eq!(doc.diagnostics()[0].code, "engine.panic");
    assert_eq!(render_parsed(&doc).diagnostics[0].code, "engine.panic");
}

#[test]
fn hook_order_is_deterministic() {
    struct TraceHook {
        name: &'static str,
        sink: Arc<Mutex<Vec<String>>>,
    }

    impl RenderHook for TraceHook {
        fn before_document_parse(
            &self,
            _input: &mut RenderInput,
            _state: &mut RenderState,
            _context: &mut RenderContext,
        ) {
            self.sink
                .lock()
                .unwrap()
                .push(format!("{}:before_document_parse", self.name));
        }

        fn after_output_finalize(
            &self,
            _document: &Document,
            _output: &mut RenderOutput,
            _state: &mut RenderState,
            _context: &mut RenderContext,
        ) {
            self.sink
                .lock()
                .unwrap()
                .push(format!("{}:after_output_finalize", self.name));
        }
    }

    let sink = Arc::new(Mutex::new(Vec::new()));
    let engine = OrihaEngine::default()
        .add_hook(TraceHook {
            name: "first",
            sink: Arc::clone(&sink),
        })
        .add_hook(TraceHook {
            name: "second",
            sink: Arc::clone(&sink),
        });

    let _ = engine.render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    let events = sink.lock().unwrap().clone();
    assert_eq!(
        events,
        vec![
            "first:before_document_parse".to_string(),
            "second:before_document_parse".to_string(),
            "first:after_output_finalize".to_string(),
            "second:after_output_finalize".to_string(),
        ]
    );
}

#[test]
fn hook_mutation_is_reflected_in_output() {
    struct MutatingHook;

    impl RenderHook for MutatingHook {
        fn before_block_render(
            &self,
            _block: &mut BlockNode,
            fragment: &mut RenderFragment,
            state: &mut RenderState,
            _context: &RenderContext,
        ) {
            fragment.attrs.insert("hook".to_string(), "on".to_string());
            state.diagnostics.push(RenderDiagnostic {
                severity: Severity::Info,
                code: "hook.info".to_string(),
                message: "hook touched block".to_string(),
                source_range: None,
                block_id: None,
                engine: DiagnosticEngine::Core,
            });
        }
    }

    let output = OrihaEngine::default()
        .add_hook(MutatingHook)
        .render(RenderInput {
            markdown: "text".to_string(),
            mode: RenderMode::StyledHtml,
            ..RenderInput::default_for_test()
        });

    assert!(output.html.contains("data-oriha-hook=\"on\""));
    assert!(output.diagnostics.iter().any(|d| d.code == "hook.info"));
}

#[test]
fn render_json_handles_invalid_input() {
    let output = render_json("{");
    assert!(output.contains("input.invalid_json"));
}

#[test]
fn stats_redacts_unknown_code_language_kinds() {
    let output = render(RenderInput {
        markdown: "```john@example.com\nsecret\n```\n\n```rust token-value\nlet value = 1;\n```\n"
            .to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });
    let stats_json = serde_json::to_string(&output.stats).expect("stats serialize");

    assert_eq!(
        kind_count(&output.stats.code_blocks_by_language_kind, "other"),
        1
    );
    assert_eq!(
        kind_count(&output.stats.code_blocks_by_language_kind, "rust"),
        1
    );
    assert!(!stats_json.contains("john@example.com"));
    assert!(!stats_json.contains("token-value"));
}

#[test]
fn stats_redacts_custom_callout_kinds() {
    let output = render(RenderInput {
        markdown: ":::customer-escalation\nBody\n:::\n".to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });
    let stats_json = serde_json::to_string(&output.stats).expect("stats serialize");

    assert_eq!(kind_count(&output.stats.callouts_by_kind, "other"), 1);
    assert!(!stats_json.contains("customer-escalation"));
}

#[test]
fn stats_redacts_unknown_error_diagnostic_codes() {
    struct ErrorHook;

    impl RenderHook for ErrorHook {
        fn before_document_parse(
            &self,
            _input: &mut RenderInput,
            state: &mut RenderState,
            _context: &mut RenderContext,
        ) {
            state.diagnostics.push(RenderDiagnostic {
                severity: Severity::Error,
                code: "customer.secret_code".to_string(),
                message: "internal-secret-message".to_string(),
                source_range: None,
                block_id: None,
                engine: DiagnosticEngine::Core,
            });
        }
    }

    let output = OrihaEngine::default()
        .add_hook(ErrorHook)
        .render(RenderInput {
            markdown: "text".to_string(),
            mode: RenderMode::SemanticHtml,
            ..RenderInput::default_for_test()
        });
    let stats_json = serde_json::to_string(&output.stats).expect("stats serialize");

    assert_eq!(kind_count(&output.stats.issues_by_kind, "unknownError"), 1);
    assert!(!stats_json.contains("customer.secret_code"));
    assert!(!stats_json.contains("internal-secret-message"));
}

#[test]
fn max_source_bytes_limit_reports_error() {
    let output = OrihaEngine::default()
        .with_limits(EngineLimits {
            max_source_bytes: 3,
            ..EngineLimits::default()
        })
        .render(RenderInput {
            markdown: "hello".to_string(),
            ..RenderInput::default_for_test()
        });

    assert!(output
        .diagnostics
        .iter()
        .any(|d| d.code == "engine.max_source_bytes_exceeded"));
}

fn kind_count(entries: &[crate::types::KindCount], kind: &str) -> u32 {
    entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.count)
        .unwrap_or(0)
}

#[test]
fn dangerous_url_scheme_is_sanitized() {
    let output = render(RenderInput {
        markdown: "[x](javascript:alert(1))".to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains("href=\"#\""));
}

#[test]
fn parser_owned_link_references_resolve_before_rendering() {
    let output = OrihaEngine::default()
        .with_unwrapped_output()
        .render(RenderInput {
            markdown: "[ΑΓΩ]: /φου \"ti\\*tle\"\n\n[αγω]\n".to_string(),
            mode: RenderMode::SemanticHtml,
            ..RenderInput::default_for_test()
        });

    assert!(output
        .html
        .contains("<a href=\"/%CF%86%CE%BF%CF%85\" title=\"ti*tle\">αγω</a>"));
}

#[test]
fn gfm_table_renders_scroll_wrapper_without_changing_table_semantics() {
    let output = OrihaEngine::default()
        .with_unwrapped_output()
        .render(RenderInput {
            markdown: "| Name | Count |\n| :--- | ---: |\n| *A* | `1` |\n".to_string(),
            mode: RenderMode::SemanticHtml,
            ..RenderInput::default_for_test()
        });

    assert!(output.html.contains(
        r#"<div class="oriha-table-scroll-wrapper" data-oriha-block-kind="table"><table class="oriha-table">"#
    ));
    assert!(output.html.contains(r#"<th align="left">Name</th>"#));
    assert!(output.html.contains(r#"<th align="right">Count</th>"#));
    assert!(output.html.contains(r#"<td align="left"><em>A</em></td>"#));
    assert!(output
        .html
        .contains(r#"<td align="right"><code>1</code></td>"#));
    assert!(output
        .stats
        .blocks_by_kind
        .iter()
        .any(|entry| entry.kind == "gfmTable" && entry.count == 1));
    assert!(output.source_map.iter().any(|block| block.kind == "table"));
}

#[test]
fn gfm_task_list_marks_parent_list_for_compact_spacing() {
    let output = OrihaEngine::default()
        .with_unwrapped_output()
        .render(RenderInput {
            markdown: "- [ ] Draft\n- [x] Review\n".to_string(),
            mode: RenderMode::SemanticHtml,
            ..RenderInput::default_for_test()
        });

    assert!(output
        .html
        .contains(r#"class="oriha-list oriha-task-list""#));
    assert!(output
        .html
        .contains(r#"class="oriha-list-item oriha-task-list-item""#));
}

#[test]
fn styled_task_list_uses_css_checkbox_instead_of_native_input() {
    let output = OrihaEngine::default()
        .with_unwrapped_output()
        .render(RenderInput {
            markdown: "- [ ] Draft\n- [x] Review\n".to_string(),
            mode: RenderMode::StyledHtml,
            ..RenderInput::default_for_test()
        });

    assert!(!output.html.contains(r#"<input"#));
    assert!(output.html.contains(
        r#"<span class="oriha-task-checkbox" role="checkbox" aria-checked="false" aria-disabled="true"></span> Draft"#
    ));
    assert!(output.html.contains(
        r#"<span class="oriha-task-checkbox oriha-task-checkbox--checked" role="checkbox" aria-checked="true" aria-disabled="true"></span> Review"#
    ));
}

#[test]
fn semantic_task_list_keeps_native_checkbox() {
    let output = OrihaEngine::default()
        .with_unwrapped_output()
        .render(RenderInput {
            markdown: "- [ ] Draft\n- [x] Review\n".to_string(),
            mode: RenderMode::SemanticHtml,
            ..RenderInput::default_for_test()
        });

    assert!(output
        .html
        .contains(r#"<input disabled="" type="checkbox"> Draft"#));
    assert!(output
        .html
        .contains(r#"<input checked="" disabled="" type="checkbox"> Review"#));
}

#[test]
fn raw_html_table_is_not_wrapped_as_gfm_table() {
    let output = OrihaEngine::default()
        .with_unwrapped_output()
        .render(RenderInput {
            markdown: "<table><tr><td>raw</td></tr></table>\n".to_string(),
            mode: RenderMode::SemanticHtml,
            features: FeatureFlags {
                raw_html: true,
                ..FeatureFlags::default()
            },
            ..RenderInput::default_for_test()
        });

    assert!(output.html.contains("<table><tr><td>raw</td></tr></table>"));
    assert!(!output.html.contains("oriha-table-scroll-wrapper"));
    assert!(!output.html.contains("oriha-table"));
}

#[test]
fn watermark_renders_html_node_only_when_requested() {
    let without_watermark = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::StyledHtml,
        ..RenderInput::default_for_test()
    });
    assert!(!without_watermark.html.contains("oriha-watermark"));

    let with_watermark = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::StyledHtml,
        render_options: RenderOptions {
            watermark: Some(WatermarkOptions {
                text: "Preview <draft>".to_string(),
                image_src: Some("https://example.com/qr?a=1&b=<tag>".to_string()),
                opacity: None,
            }),
            ..RenderOptions::default()
        },
        ..RenderInput::default_for_test()
    });

    assert!(with_watermark.html.contains(r#"class="oriha-watermark""#));
    assert!(with_watermark.html.contains(
        r#"<img src="https://example.com/qr?a=1&amp;b=&lt;tag&gt;" alt="" class="oriha-watermark__image" />"#
    ));
    assert!(with_watermark
        .html
        .contains(r#"<span class="oriha-watermark__text">Preview &lt;draft&gt;</span>"#));
    assert!(with_watermark
        .css
        .as_deref()
        .is_some_and(|css| css.contains(".oriha-watermark__image")));
}

#[test]
fn watermark_omits_image_when_image_src_is_absent() {
    let output = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::StyledHtml,
        render_options: RenderOptions {
            watermark: Some(WatermarkOptions {
                text: "Preview".to_string(),
                image_src: None,
                opacity: None,
            }),
            ..RenderOptions::default()
        },
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains(r#"class="oriha-watermark""#));
    assert!(!output.html.contains("oriha-watermark__image"));
}

#[test]
fn watermark_opacity_overrides_styled_css_when_present() {
    let output = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::StyledHtml,
        render_options: RenderOptions {
            watermark: Some(WatermarkOptions {
                text: "Preview".to_string(),
                image_src: None,
                opacity: Some(1.4),
            }),
            ..RenderOptions::default()
        },
        ..RenderInput::default_for_test()
    });

    let css = output.css.expect("styled output should include CSS");
    assert!(css.contains("--oriha-watermark-opacity: 0.45;"));
    assert!(css.ends_with(":root {\n  --oriha-watermark-opacity: 1;\n}\n"));
}

#[test]
fn watermark_absent_opacity_keeps_theme_default_css() {
    let output = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::StyledHtml,
        render_options: RenderOptions {
            watermark: Some(WatermarkOptions {
                text: "Preview".to_string(),
                image_src: None,
                opacity: None,
            }),
            ..RenderOptions::default()
        },
        ..RenderInput::default_for_test()
    });

    let css = output.css.expect("styled output should include CSS");
    assert!(css.contains("--oriha-watermark-opacity: 0.45;"));
    assert!(!css.ends_with(":root {\n  --oriha-watermark-opacity: 0.45;\n}\n"));
}

#[test]
fn watermark_opacity_does_not_create_semantic_css() {
    let output = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::SemanticHtml,
        render_options: RenderOptions {
            watermark: Some(WatermarkOptions {
                text: "Preview".to_string(),
                image_src: None,
                opacity: Some(0.25),
            }),
            ..RenderOptions::default()
        },
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains(r#"class="oriha-watermark""#));
    assert!(output.css.is_none());
}

#[test]
fn styled_mode_returns_css_and_semantic_mode_does_not() {
    let styled = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::StyledHtml,
        ..RenderInput::default_for_test()
    });
    let semantic = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default_for_test()
    });

    assert!(styled.css.is_some());
    assert!(semantic.css.is_none());
}

#[test]
fn appearance_style_is_resolved_for_wrapper_and_css() {
    let output = render(RenderInput {
        markdown: "hello".to_string(),
        mode: RenderMode::StyledHtml,
        appearance: AppearanceOptions {
            style: "simple".to_string(),
            body_font_family: FontFamilySpec {
                family: "ExampleSans".to_string(),
                fallbacks: Vec::new(),
            },
            ..AppearanceOptions::default()
        },
        ..RenderInput::default_for_test()
    });

    assert!(output.html.contains(r#"data-oriha-theme="simple-light""#));
    assert!(output
        .css
        .as_deref()
        .is_some_and(|css| css.contains("ExampleSans")));
}

#[test]
fn appearance_changes_do_not_alter_rendered_block_or_source_map_structure() {
    let markdown = "# Title\n\nParagraph with [link](https://example.com).\n";
    let baseline = render(RenderInput {
        markdown: markdown.to_string(),
        mode: RenderMode::StyledHtml,
        ..RenderInput::default_for_test()
    });
    let appearance_changed = render(RenderInput {
        markdown: markdown.to_string(),
        mode: RenderMode::StyledHtml,
        appearance: AppearanceOptions {
            style: "alternate".to_string(),
            ..AppearanceOptions::default()
        },
        ..RenderInput::default_for_test()
    });

    let baseline_blocks = baseline
        .source_map
        .iter()
        .map(|block| (&block.kind, &block.source_range, &block.metadata))
        .collect::<Vec<_>>();
    let appearance_blocks = appearance_changed
        .source_map
        .iter()
        .map(|block| (&block.kind, &block.source_range, &block.metadata))
        .collect::<Vec<_>>();

    assert_eq!(baseline_blocks.len(), appearance_blocks.len());
    for (baseline_block, appearance_block) in baseline_blocks.iter().zip(appearance_blocks) {
        assert_eq!(baseline_block.0, appearance_block.0);
        assert_eq!(baseline_block.1.start_line, appearance_block.1.start_line);
        assert_eq!(
            baseline_block.1.start_column,
            appearance_block.1.start_column
        );
        assert_eq!(baseline_block.1.end_line, appearance_block.1.end_line);
        assert_eq!(baseline_block.1.end_column, appearance_block.1.end_column);
        assert_eq!(baseline_block.2, appearance_block.2);
    }
}

#[test]
fn unwrapped_output_mode_returns_fragment_only() {
    let output = OrihaEngine::default()
        .with_unwrapped_output()
        .render(RenderInput {
            markdown: "hello".to_string(),
            mode: RenderMode::SemanticHtml,
            ..RenderInput::default_for_test()
        });

    assert!(!output.html.contains("<article"));
}

#[derive(Deserialize)]
struct CommonMarkExample {
    markdown: String,
    html: String,
    section: String,
}

#[test]
fn commonmark_0_31_2_engine_fixture_all_examples_must_match() {
    let fixture = include_str!("../../tests/fixtures/commonmark_0_31_2_spec.json");
    let examples: Vec<CommonMarkExample> =
        serde_json::from_str(fixture).expect("valid CommonMark fixture");

    let mut mismatches: Vec<(usize, String, String, String)> = Vec::new();

    for (index, example) in examples.iter().enumerate() {
        let actual = SpecHtmlRenderer::render(&example.markdown);
        let normalized_actual = normalize_html_for_compare(&actual);
        let normalized_expected = normalize_html_for_compare(&example.html);
        if normalized_actual != normalized_expected {
            mismatches.push((
                index,
                example.section.clone(),
                normalized_expected,
                normalized_actual,
            ));
        }
    }

    if !mismatches.is_empty() {
        let mut details = String::new();
        let preview = mismatches.len().min(20);
        for (index, section, expected, actual) in mismatches.iter().take(preview) {
            let _ = std::fmt::Write::write_fmt(
                &mut details,
                format_args!(
                    "\n- index={index}, section={section}\n  expected={expected:?}\n  actual={actual:?}"
                ),
            );
        }
        panic!(
            "CommonMark 0.31.2 conformance mismatch: {}/{} examples failed. First {} mismatches:{}",
            mismatches.len(),
            examples.len(),
            preview,
            details
        );
    }
}

#[test]
fn commonmark_0_31_2_engine_fixture_has_at_least_one_match() {
    let fixture = include_str!("../../tests/fixtures/commonmark_0_31_2_spec.json");
    let examples: Vec<CommonMarkExample> =
        serde_json::from_str(fixture).expect("valid CommonMark fixture");
    let mut pass_count = 0usize;

    for example in &examples {
        let actual = SpecHtmlRenderer::render(&example.markdown);
        if normalize_html_for_compare(&actual) == normalize_html_for_compare(&example.html) {
            pass_count += 1;
        }
    }
    println!(
        "CommonMark matches with current engine: {}/{}",
        pass_count,
        examples.len()
    );

    assert!(
        pass_count > 0,
        "CommonMark fixture had zero matches (0/{})",
        examples.len()
    );
}

#[test]
fn gfm_0_29_extension_fixture_covers_target_sections() {
    let fixture = include_str!("../../tests/fixtures/gfm_0_29_extension_spec.json");
    let examples: Vec<CommonMarkExample> =
        serde_json::from_str(fixture).expect("valid GFM extension fixture");
    let sections = examples
        .iter()
        .map(|example| example.section.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(examples.len(), 28);
    assert!(sections.contains("Tables (extension)"));
    assert!(sections.contains("Task list items (extension)"));
    assert!(sections.contains("Strikethrough (extension)"));
    assert!(sections.contains("Autolinks (extension)"));
    assert!(sections.contains("Disallowed Raw HTML (extension)"));
}

#[test]
fn gfm_0_29_extension_fixture_has_at_least_one_match() {
    let fixture = include_str!("../../tests/fixtures/gfm_0_29_extension_spec.json");
    let examples: Vec<CommonMarkExample> =
        serde_json::from_str(fixture).expect("valid GFM extension fixture");
    let mut pass_count = 0usize;

    for example in &examples {
        let actual = SpecHtmlRenderer::render_gfm(&example.markdown);
        if normalize_html_for_compare(&actual) == normalize_html_for_compare(&example.html) {
            pass_count += 1;
        }
    }
    println!(
        "GFM extension matches with current engine: {}/{}",
        pass_count,
        examples.len()
    );

    assert!(
        pass_count > 0,
        "GFM extension fixture had zero matches (0/{})",
        examples.len()
    );
}

#[test]
fn gfm_extended_autolink_examples_match() {
    let examples: Vec<CommonMarkExample> = serde_json::from_str(include_str!(
        "../../tests/fixtures/gfm_0_29_extension_spec.json"
    ))
    .unwrap();
    let mut count = 0;
    for example in examples
        .iter()
        .filter(|e| e.section == "Autolinks (extension)")
    {
        count += 1;
        assert_eq!(
            normalize_html_for_compare(&SpecHtmlRenderer::render_gfm(&example.markdown)),
            normalize_html_for_compare(&example.html),
            "{}",
            example.markdown
        );
    }
    assert_eq!(count, 14);
}

#[test]
#[ignore = "Enable when the renderer claims full GFM extension conformance."]
fn gfm_0_29_extension_fixture_all_examples_must_match() {
    let fixture = include_str!("../../tests/fixtures/gfm_0_29_extension_spec.json");
    let examples: Vec<CommonMarkExample> =
        serde_json::from_str(fixture).expect("valid GFM extension fixture");
    let mut mismatches: Vec<(usize, String, String, String)> = Vec::new();

    for (index, example) in examples.iter().enumerate() {
        let actual = SpecHtmlRenderer::render_gfm(&example.markdown);
        let normalized_actual = normalize_html_for_compare(&actual);
        let normalized_expected = normalize_html_for_compare(&example.html);
        if normalized_actual != normalized_expected {
            mismatches.push((
                index,
                example.section.clone(),
                normalized_expected,
                normalized_actual,
            ));
        }
    }

    if !mismatches.is_empty() {
        let mut details = String::new();
        let preview = mismatches.len().min(20);
        for (index, section, expected, actual) in mismatches.iter().take(preview) {
            let _ = std::fmt::Write::write_fmt(
                &mut details,
                format_args!(
                    "\n- index={index}, section={section}\n  expected={expected:?}\n  actual={actual:?}"
                ),
            );
        }
        panic!(
            "GFM 0.29 extension conformance mismatch: {}/{} examples failed. First {} mismatches:{}",
            mismatches.len(),
            examples.len(),
            preview,
            details
        );
    }
}

fn normalize_html_for_compare(value: &str) -> String {
    let mut normalized = value
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'");

    while let Some(start) = normalized.find(">\n<") {
        normalized.replace_range(start + 1..start + 2, "");
    }

    while let Some(start) = normalized.find(">\r\n<") {
        normalized.replace_range(start + 1..start + 3, "");
    }

    normalized
}

#[test]
fn both_block_modes_reuse_actual_block_and_inline_parser_calls() {
    crate::parser::PARSE_CALLS.with(|calls| calls.set((0, 0)));
    let parsed = parse(RenderInput {
        markdown: "# Reuse **both**\n\n> nested `code`\n\n[^f]\n\n[^f]: footnote".into(),
        ..RenderInput::default_for_test()
    });
    let counts = crate::parser::PARSE_CALLS.with(|calls| calls.get());
    assert!(counts.0 > 0 && counts.1 > 0);
    for _ in 0..2 {
        for mode in [
            crate::BlockOutputMode::Structured,
            crate::BlockOutputMode::Rendered,
        ] {
            let output = derive_parsed_blocks(&parsed, mode);
            assert_eq!(serde_json::to_value(output).unwrap()["status"], "complete");
            assert_eq!(crate::parser::PARSE_CALLS.with(|calls| calls.get()), counts);
        }
    }
}

#[test]
fn inline_renderer_context_and_context_free_paths_share_raw_html_policy() {
    let source = "Before **<img src=x onerror=alert(1)>** <script>code</script> after";
    let engine = OrihaEngine::default();
    for raw_html in [false, true] {
        for gfm in [false, true] {
            let features = FeatureFlags {
                raw_html,
                gfm,
                ..Default::default()
            };
            let parsed = engine.parse(RenderInput {
                markdown: "context".into(),
                features: features.clone(),
                ..Default::default()
            });
            let context = parsed.context.borrow(
                &engine.hooks,
                engine.inline_renderer.as_ref(),
                parsed.limits.max_output_bytes,
            );
            let actual = engine.inline_renderer.render_with_context(
                source,
                &context,
                &mut RenderStats::default(),
            );
            let expected =
                engine
                    .inline_renderer
                    .render(source, &features, &mut RenderStats::default());
            assert_eq!(actual, expected);
            assert_eq!(actual.contains("<img src=x onerror=alert(1)>"), raw_html);
            assert_eq!(actual.contains("<script>"), raw_html && !gfm);
        }
    }
}
