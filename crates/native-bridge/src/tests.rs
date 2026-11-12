use super::*;

fn custom_input(plan: Plan) -> RenderInput {
    RenderInput {
        markdown:
            ":::note\nCallout\n:::\n\n```plantuml\n@startuml\nAlice -> Bob: Hello\n@enduml\n```\n"
                .to_string(),
        plan,
        rendering_options: RenderingOptions {
            syntax_theme: SyntaxTheme::Custom,
            custom_syntax_theme: Some("monokai".to_string()),
            heading_font: HeadingFont::Custom,
            custom_heading_font: Some(FontFamilySpec {
                family: "\"Bridge Heading\"".to_string(),
                fallbacks: vec!["serif".to_string()],
            }),
            body_font: BodyFont::Custom,
            custom_body_font: Some(FontFamilySpec {
                family: "\"Bridge Body\"".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            }),
            code_font: CodeFont::Custom,
            custom_code_font: Some(FontFamilySpec {
                family: "\"Bridge Code\"".to_string(),
                fallbacks: vec!["monospace".to_string()],
            }),
            ..RenderingOptions::default()
        },
        ..RenderInput::default()
    }
}

fn render_input(input: RenderInput) -> RenderOutput {
    let prepared = prepare_markdown_layout(input);
    render_prepared_markdown(prepared, TextMeasurementResults::default())
}

#[test]
fn exposes_native_bridge_version_info() {
    let info = native_bridge_version_info();

    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    assert!(!info.git_short_hash.is_empty());
}

#[test]
fn json_bridge_prepare_and_render_round_trips() {
    let input = RenderInput {
        markdown: "# JSON bridge\n\nBody\n".to_string(),
        ..RenderInput::default()
    };
    let input_json = serde_json::to_string(&input).expect("input should serialize");

    let prepared_json = prepare_markdown_layout_json(input_json);
    let prepared: PreparedMarkdownLayout =
        serde_json::from_str(&prepared_json).expect("prepared layout should deserialize");
    assert_eq!(prepared.input.markdown, input.markdown);

    let output_json = render_prepared_markdown_json(prepared_json, "[]".to_string());
    let output: RenderOutput =
        serde_json::from_str(&output_json).expect("render output should deserialize");
    assert!(output.html.contains("JSON bridge"), "{}", output.html);
}

#[test]
fn json_bridge_resolves_settings() {
    let input = BridgeSettingsInput {
        locale: Some("ja".to_string()),
        ..BridgeSettingsInput::default()
    };
    let input_json = serde_json::to_string(&input).expect("settings input should serialize");

    let settings_json = resolve_render_settings_json(input_json);
    let settings: BridgeResolvedSettings =
        serde_json::from_str(&settings_json).expect("settings should deserialize");

    assert_eq!(settings.resolved_locale, "ja");
    assert_eq!(settings.effective_input.locale, "ja");
}

#[test]
fn render_stats_json_contract_is_bridge_stable() {
    let stats = RenderStats {
        schema_version: 1,
        total_blocks: 3,
        blocks_by_kind: vec![KindCount {
            kind: "heading".to_string(),
            count: 1,
        }],
        headings_by_level: vec![KindCount {
            kind: "h1".to_string(),
            count: 1,
        }],
        code_blocks_by_language_kind: vec![KindCount {
            kind: "rust".to_string(),
            count: 1,
        }],
        callouts_by_kind: vec![KindCount {
            kind: "note".to_string(),
            count: 1,
        }],
        locked_features_by_kind: vec![KindCount {
            kind: "plantuml".to_string(),
            count: 1,
        }],
        unsupported_by_kind: vec![KindCount {
            kind: "plantumlDiagram".to_string(),
            count: 1,
        }],
        issues_by_kind: vec![KindCount {
            kind: "unknownError".to_string(),
            count: 1,
        }],
    };

    let value = serde_json::to_value(&stats).expect("bridge stats should serialize");

    assert_eq!(
        value,
        serde_json::json!({
            "schemaVersion": 1,
            "totalBlocks": 3,
            "blocksByKind": [{ "kind": "heading", "count": 1 }],
            "headingsByLevel": [{ "kind": "h1", "count": 1 }],
            "codeBlocksByLanguageKind": [{ "kind": "rust", "count": 1 }],
            "calloutsByKind": [{ "kind": "note", "count": 1 }],
            "lockedFeaturesByKind": [{ "kind": "plantuml", "count": 1 }],
            "unsupportedByKind": [{ "kind": "plantumlDiagram", "count": 1 }],
            "issuesByKind": [{ "kind": "unknownError", "count": 1 }]
        })
    );

    let decoded: RenderStats =
        serde_json::from_value(value).expect("bridge stats JSON should deserialize");
    assert_eq!(decoded.schema_version, 1);
    assert_eq!(kind_count(&decoded.locked_features_by_kind, "plantuml"), 1);
}

#[test]
fn usage_analytics_outbox_typed_start_enqueue_returns_step() {
    let input = NativeUsageAnalyticsOutboxEnqueueStartInput {
        config: NativeUsageAnalyticsOutboxConfig {
            ingest_url: "https://stats.example.test/api/ingest".to_string(),
            api_key: "test-key".to_string(),
            request_timeout_ms: 5_000,
            max_queued_events: 100,
            retention_ms: 86_400_000,
        },
        render_payload: NativeUsageAnalyticsRenderPayloadInput {
            platform: "iOS".to_string(),
            app_version: "1.0".to_string(),
            build_version: "42".to_string(),
            locale: "ja".to_string(),
            subscription_plan: "free".to_string(),
            stats: RenderStats {
                schema_version: 1,
                total_blocks: 1,
                blocks_by_kind: vec![KindCount {
                    kind: "paragraph".to_string(),
                    count: 1,
                }],
                ..RenderStats::default()
            },
            event_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        },
        is_delivery_enabled: true,
        delivery_generation: 1,
    };

    let step = usage_analytics_outbox_start_enqueue(input, 1_000);
    assert!(!step.is_complete);
    let command = step.commands.first().expect("write command");
    assert_eq!(
        command.kind,
        NativeUsageAnalyticsOutboxCommandKind::WriteEnvelope
    );
    let envelope_json = command.envelope_json.as_deref().expect("envelope");
    let envelope: serde_json::Value = serde_json::from_str(envelope_json).expect("envelope json");
    let payload: serde_json::Value =
        serde_json::from_str(envelope["payloadJson"].as_str().expect("payload json"))
            .expect("payload");
    assert_eq!(payload["eventId"], "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(payload["platform"], "iOS");
    assert_eq!(payload["stats"]["blocksByKind"][0]["kind"], "paragraph");
}

#[test]
fn free_disables_plantuml_and_clamps_custom_theme_and_fonts() {
    let output = render_input(custom_input(Plan::Free));

    assert!(!output.html.contains(r#"class="oriha-plantuml""#));
    assert!(output.html.contains(r#"class="language-plantuml""#));
    assert!(!output.css.unwrap_or_default().contains("Bridge Heading"));
    assert_bridge_locked_diagnostics_at_least(&output.diagnostics, 5);
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "syntaxTheme"),
        1
    );
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "headingFont"),
        1
    );
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "bodyFont"),
        1
    );
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "codeFont"),
        1
    );
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "callout"),
        1
    );
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "plantuml"),
        1
    );
}

#[test]
fn free_clamps_custom_theme_and_fonts_to_plan_defaults() {
    let prepared = prepare_markdown_layout(RenderInput {
        markdown: "# Catalog policy\n".to_string(),
        plan: Plan::Free,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Dark,
            syntax_theme: SyntaxTheme::TokyoNight,
            heading_font: HeadingFont::Custom,
            custom_heading_font: Some(FontFamilySpec {
                family: "\"Bridge Heading\"".to_string(),
                fallbacks: vec!["serif".to_string()],
            }),
            body_font: BodyFont::Custom,
            custom_body_font: Some(FontFamilySpec {
                family: "\"Bridge Body\"".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            }),
            code_font: CodeFont::Custom,
            custom_code_font: Some(FontFamilySpec {
                family: "\"Bridge Code\"".to_string(),
                fallbacks: vec!["monospace".to_string()],
            }),
            ..RenderingOptions::default()
        },
        ..RenderInput::default()
    });

    assert_eq!(
        prepared.input.rendering_options.syntax_theme,
        SyntaxTheme::GitHubDark
    );
    assert_eq!(
        prepared.input.rendering_options.heading_font,
        HeadingFont::SystemDefault
    );
    assert_eq!(
        prepared.input.rendering_options.body_font,
        BodyFont::SystemDefault
    );
    assert_eq!(
        prepared.input.rendering_options.code_font,
        CodeFont::SystemDefault
    );
    assert!(prepared
        .input
        .rendering_options
        .custom_heading_font
        .is_none());
    assert!(prepared.input.rendering_options.custom_body_font.is_none());
    assert!(prepared.input.rendering_options.custom_code_font.is_none());
    assert_bridge_locked_diagnostics_at_least(&prepared.policy_diagnostics, 4);
}

#[test]
fn body_font_size_is_available_on_free_plan_and_reaches_core_css() {
    let output = render_input(RenderInput {
        markdown: "# Sized\n\nBody\n".to_string(),
        plan: Plan::Free,
        rendering_options: RenderingOptions {
            body_font_size_px: Some(20.0),
            ..RenderingOptions::default()
        },
        ..RenderInput::default()
    });

    let css = output.css.expect("css output");
    assert!(css.contains("--oriha-font-size-body: 20px;"));
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "bodyFontSizePx"),
        0
    );
}

#[test]
fn emoji_shortcodes_render_for_all_plans_without_locked_stats() {
    for plan in [Plan::Free, Plan::Premium] {
        let output = render_input(RenderInput {
            markdown: "Hello :+1: :thumbs_up_light_skin_tone: :rocket:\n".to_string(),
            plan,
            ..RenderInput::default()
        });

        assert!(output.html.contains("Hello 👍 👍🏻 🚀"), "{}", output.html);
        assert!(
            output.stats.locked_features_by_kind.is_empty(),
            "{:?}",
            output.stats.locked_features_by_kind
        );
    }
}

#[test]
fn premium_enables_callouts_and_plantuml() {
    let mut input = RenderInput {
        markdown:
            ":::note\nCallout\n:::\n\n```plantuml\n@startuml\nAlice -> Bob: Hello\n@enduml\n```\n"
                .to_string(),
        plan: Plan::Premium,
        render_mode: RenderMode::SemanticHtml,
        ..RenderInput::default()
    };
    let output = render_input(input.clone());
    assert!(output.html.contains(r#"data-oriha-block-kind="callout""#));
    assert!(output.html.contains(r#"data-oriha-callout-kind="note""#));
    assert!(output.html.contains(r#"class="oriha-plantuml""#));

    input.plan = Plan::Free;
    let free_output = render_input(input);
    assert!(!free_output
        .html
        .contains(r#"data-oriha-block-kind="callout""#));
}

#[test]
fn free_flags_display_and_fenced_latex_as_premium_math() {
    for markdown in [
        "$$\n\\frac{1}{3}\n$$\n",
        "```latex\n\\alpha + \\beta = \\gamma\n```\n",
        "~~~latex\n\\int_0^1 x^2\\,dx\n~~~\n",
    ] {
        let prepared = prepare_markdown_layout(RenderInput {
            markdown: markdown.to_string(),
            plan: Plan::Free,
            ..RenderInput::default()
        });
        assert!(
            has_bridge_locked_diagnostic(&prepared.policy_diagnostics),
            "{markdown}"
        );
        assert_eq!(
            kind_count(&prepared.policy_stats.locked_features_by_kind, "math"),
            1
        );
    }

    let inline_currency = prepare_markdown_layout(RenderInput {
        markdown: "The price is $12 today.\n".to_string(),
        plan: Plan::Free,
        ..RenderInput::default()
    });
    assert!(!has_bridge_locked_diagnostic(
        &inline_currency.policy_diagnostics
    ));
}

#[test]
fn locked_feature_stats_redact_markdown_literals() {
    let prepared = prepare_markdown_layout(RenderInput {
        markdown: [
            "```mermaid customer@example.com",
            "graph TD",
            "```",
            "~~~plantuml internal-token",
            "@startuml",
            "@enduml",
            "~~~",
            "```tex private-math",
            "\\alpha",
            "```",
        ]
        .join("\n"),
        plan: Plan::Free,
        ..RenderInput::default()
    });
    let stats_json = serde_json::to_string(&prepared.policy_stats).expect("stats should serialize");

    assert!(has_bridge_locked_diagnostic(&prepared.policy_diagnostics));
    assert_eq!(
        kind_count(&prepared.policy_stats.locked_features_by_kind, "mermaid"),
        1
    );
    assert_eq!(
        kind_count(&prepared.policy_stats.locked_features_by_kind, "plantuml"),
        1
    );
    assert_eq!(
        kind_count(&prepared.policy_stats.locked_features_by_kind, "math"),
        1
    );
    assert!(!stats_json.contains("customer@example.com"));
    assert!(!stats_json.contains("internal-token"));
    assert!(!stats_json.contains("private-math"));
}

#[test]
fn raw_html_is_available_on_all_plans() {
    for plan in [Plan::Free, Plan::Premium] {
        let resolved = resolve_render_settings(BridgeSettingsInput {
            plan: plan.clone(),
            ..BridgeSettingsInput::default()
        });
        assert!(resolved
            .features
            .iter()
            .any(|feature| matches!(feature, BridgeFeature::RawHtml)));

        let output = render_input(RenderInput {
            markdown: "<div data-sample=\"raw-html\"><span>raw</span></div>\n".to_string(),
            plan,
            render_mode: RenderMode::SemanticHtml,
            ..RenderInput::default()
        });

        assert!(output.html.contains(r#"<div data-sample="raw-html">"#));
        assert!(output.html.contains(r#"<span>raw</span>"#));
        assert!(!output.html.contains("&lt;div"));
    }
}

#[test]
fn premium_preserves_custom_theme_fonts_and_plantuml() {
    let output = render_input(custom_input(Plan::Premium));

    assert!(output.html.contains(r#"class="oriha-plantuml""#));
    let css = output.css.unwrap_or_default();
    assert!(css.contains("Bridge Heading"));
    assert!(css.contains("Bridge Body"));
    assert!(css.contains("Bridge Code"));
    assert!(!has_bridge_locked_diagnostic(&output.diagnostics));
}

#[test]
fn prepared_layout_preserves_policy_diagnostics_for_final_render() {
    let prepared = prepare_markdown_layout(custom_input(Plan::Free));
    assert_bridge_locked_diagnostics_at_least(&prepared.policy_diagnostics, 5);
    assert_eq!(
        prepared.input.rendering_options.syntax_theme,
        SyntaxTheme::GitHubLight
    );
    assert_eq!(
        prepared.input.rendering_options.heading_font,
        HeadingFont::SystemDefault
    );
    assert_eq!(
        kind_count(&prepared.policy_stats.locked_features_by_kind, "plantuml"),
        1
    );

    let output = render_prepared_markdown(prepared, TextMeasurementResults::default());
    assert_bridge_locked_diagnostics_at_least(&output.diagnostics, 5);
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "plantuml"),
        1
    );
    assert_eq!(
        kind_count(&output.stats.locked_features_by_kind, "callout"),
        1
    );
}

#[test]
fn premium_prepared_layout_preserves_custom_selection_and_measurements() {
    let prepared = prepare_markdown_layout(custom_input(Plan::Premium));

    assert_eq!(
        prepared.input.rendering_options.syntax_theme,
        SyntaxTheme::Custom
    );
    assert_eq!(
        prepared
            .input
            .rendering_options
            .custom_syntax_theme
            .as_deref(),
        Some("monokai")
    );
    assert_eq!(
        prepared.input.rendering_options.heading_font,
        HeadingFont::Custom
    );
    assert!(!prepared.text_measurements.requests.is_empty());
    assert!(!has_bridge_locked_diagnostic(&prepared.policy_diagnostics));
}

#[test]
fn print_options_are_forwarded_to_core_html() {
    let output = render_input(RenderInput {
        markdown: "# Print\n".to_string(),
        print: Some(PrintOptions {
            page_size: Some("A4".to_string()),
            margin: Some("12mm".to_string()),
        }),
        ..RenderInput::default()
    });

    let css = output.css.unwrap_or_default();
    assert!(css.contains("@page"));
    assert!(css.contains("size: A4"));
    assert!(css.contains("margin: 12mm"));
}

#[test]
fn free_print_export_adds_default_watermark() {
    let output = render_input(RenderInput {
        markdown: "# Free PDF\n".to_string(),
        plan: Plan::Free,
        print: Some(PrintOptions {
            page_size: Some("A4".to_string()),
            margin: Some("12mm".to_string()),
        }),
        ..RenderInput::default()
    });

    assert!(output.html.contains(r#"class="oriha-watermark""#));
    assert!(output.html.contains(oriha_core::DEFAULT_WATERMARK_TEXT));
    assert!(output.html.contains(r#"class="oriha-watermark__image""#));
    assert!(output.html.contains("data:image/svg+xml,"));
}

#[test]
fn paid_print_export_without_watermark_stays_unwatermarked() {
    let output = render_input(RenderInput {
        markdown: "# Paid PDF\n".to_string(),
        plan: Plan::Premium,
        print: Some(PrintOptions {
            page_size: Some("A4".to_string()),
            margin: Some("12mm".to_string()),
        }),
        ..RenderInput::default()
    });

    assert!(!output.html.contains(r#"class="oriha-watermark""#));
    assert!(!output.html.contains(oriha_core::DEFAULT_WATERMARK_TEXT));
}

#[test]
fn free_print_export_preserves_explicit_watermark() {
    let output = render_input(RenderInput {
        markdown: "# Branded PDF\n".to_string(),
        plan: Plan::Free,
        print: Some(PrintOptions {
            page_size: Some("A4".to_string()),
            margin: Some("12mm".to_string()),
        }),
        watermark: Some(WatermarkOptions {
            text: "Team Preview <internal>".to_string(),
            image_src: Some("https://example.com/watermark.png?draft=1&size=small".to_string()),
            opacity: Some(0.42),
        }),
        ..RenderInput::default()
    });

    assert!(output.html.contains(r#"class="oriha-watermark""#));
    assert!(output.html.contains("Team Preview &lt;internal&gt;"));
    assert!(output
        .html
        .contains(r#"src="https://example.com/watermark.png?draft=1&amp;size=small""#));
    assert!(!output.html.contains(oriha_core::DEFAULT_WATERMARK_TEXT));
    assert!(output
        .css
        .unwrap_or_default()
        .contains("--oriha-watermark-opacity: 0.42;"));
}

#[test]
fn typed_bridge_round_trips_prepared_layout_and_render_output() {
    let prepared = prepare_markdown_layout(RenderInput {
        markdown: "```plantuml\n@startuml\nAlice -> Bob: 日本語 mixed label\n@enduml\n```\n"
            .to_string(),
        plan: Plan::Premium,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Dark,
            syntax_theme: SyntaxTheme::TokyoNight,
            heading_font: HeadingFont::Custom,
            custom_heading_font: Some(FontFamilySpec {
                family: "\"Bridge Heading\"".to_string(),
                fallbacks: vec!["serif".to_string()],
            }),
            body_font: BodyFont::Custom,
            custom_body_font: Some(FontFamilySpec {
                family: "\"Bridge Body\"".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            }),
            code_font: CodeFont::Custom,
            custom_code_font: Some(FontFamilySpec {
                family: "\"Bridge Code\"".to_string(),
                fallbacks: vec!["monospace".to_string()],
            }),
            rounded_corners: false,
            shadows: false,
            full_table_data: true,
            block_borders: true,
            ..RenderingOptions::default()
        },
        ..RenderInput::default()
    });
    assert!(!prepared.text_measurements.requests.is_empty());

    let output = render_prepared_markdown(prepared, TextMeasurementResults::default());
    assert!(output.html.contains("oriha-plantuml"));
    let css = output.css.unwrap_or_default();
    assert!(css.contains("Bridge Heading"));
    assert!(css.contains("Bridge Body"));
    assert!(css.contains("Bridge Code"));
    assert!(!css.contains("fonts.googleapis.com"));
}

#[test]
fn typed_bridge_round_trips_prepared_patch_and_render_output() {
    let input = RenderInput {
        markdown: "Alpha\n\nBeta\n".to_string(),
        plan: Plan::Free,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Light,
            ..RenderingOptions::default()
        },
        ..RenderInput::default()
    };
    let prepared = prepare_markdown_patch(None, input);
    assert_eq!(prepared.version, PREPARED_MARKDOWN_PATCH_VERSION);
    assert!(prepared.previous_snapshot.is_none());
    assert!(prepared.text_measurements.requests.is_empty());

    let output = render_prepared_markdown_patch(prepared, TextMeasurementResults::default());
    assert_eq!(output.strategy, RenderPatchStrategy::ReplaceAll);
    assert!(output.output.html.contains("Alpha"));
    assert!(output.output.html.contains("Beta"));

    let input = RenderInput {
        markdown: "Alpha edited\n\nBeta\n".to_string(),
        plan: Plan::Free,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Light,
            ..RenderingOptions::default()
        },
        ..RenderInput::default()
    };
    let prepared = prepare_markdown_patch(Some(output.snapshot.clone()), input);
    assert!(prepared.previous_snapshot.is_some());

    let output = render_prepared_markdown_patch(prepared, TextMeasurementResults::default());
    assert_eq!(output.strategy, RenderPatchStrategy::Incremental);
    assert!(output
        .patches
        .iter()
        .any(|patch| patch.op == RenderPatchOperationKind::ReplaceBlocks));
    assert!(output.output.html.contains("Alpha edited"));
}

#[test]
fn typed_bridge_round_trips_prepared_html_export_and_output() {
    let prepared = prepare_html_export(
        RenderInput {
            markdown: "# Export\n\nParagraph\n".to_string(),
            plan: Plan::Premium,
            rendering_options: RenderingOptions {
                color_scheme: ColorScheme::Light,
                ..RenderingOptions::default()
            },
            ..RenderInput::default()
        },
        Some(HtmlExportOptions {
            minify: false,
            obfuscate: false,
            preserve_heading_anchors: true,
            strip_oriha_data_attributes: true,
        }),
    );
    assert_eq!(prepared.version, PREPARED_HTML_EXPORT_VERSION);
    assert!(!prepared.options.minify);
    assert!(!prepared.options.obfuscate);
    assert!(prepared.text_measurements.requests.is_empty());

    let output = render_prepared_html_export(prepared, TextMeasurementResults::default());
    assert!(output.html.contains("Export"));
    assert!(output.html.contains("Paragraph"));
    assert_eq!(
        output.stats.schema_version,
        crate::types::render_stats_schema_version()
    );
    assert!(output.diagnostics.is_empty());
}

#[test]
fn typed_html_export_uses_default_options_when_omitted() {
    let prepared = prepare_html_export(
        RenderInput {
            markdown: "# Export\n\nParagraph\n".to_string(),
            plan: Plan::Premium,
            rendering_options: RenderingOptions {
                color_scheme: ColorScheme::Light,
                ..RenderingOptions::default()
            },
            ..RenderInput::default()
        },
        None,
    );
    assert!(prepared.options.minify);
    assert!(prepared.options.obfuscate);
    assert!(prepared.options.preserve_heading_anchors);
    assert!(prepared.options.strip_oriha_data_attributes);

    let output = render_prepared_html_export(prepared, TextMeasurementResults::default());
    assert!(output.html.contains("Export"));
    assert!(!output.html.contains("data-oriha-"));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn typed_prepared_layout_forwards_export_options() {
    let prepared = prepare_markdown_layout(RenderInput {
        markdown: "# Print\n".to_string(),
        plan: Plan::Premium,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Light,
            ..RenderingOptions::default()
        },
        print: Some(PrintOptions {
            page_size: Some("A4".to_string()),
            margin: Some("12mm".to_string()),
        }),
        ..RenderInput::default()
    });
    let output = render_prepared_markdown(prepared, TextMeasurementResults::default());
    let css = output.css.unwrap_or_default();
    assert!(css.contains("@page"));
    assert!(css.contains("size: A4"));
    assert!(css.contains("margin: 12mm"));
}

#[test]
fn settings_resolver_clamps_free_look_and_feel_options() {
    let settings = settings_value(BridgeSettingsInput {
        locale: Some("ja-JP".to_string()),
        plan: Plan::Free,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Dark,
            syntax_theme: SyntaxTheme::TokyoNight,
            heading_font: HeadingFont::Custom,
            custom_heading_font: Some(FontFamilySpec {
                family: "\"Bridge Heading\"".to_string(),
                fallbacks: vec!["serif".to_string()],
            }),
            body_font: BodyFont::Custom,
            custom_body_font: Some(FontFamilySpec {
                family: "\"Bridge Body\"".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            }),
            code_font: CodeFont::Custom,
            custom_code_font: Some(FontFamilySpec {
                family: "\"Bridge Code\"".to_string(),
                fallbacks: vec!["monospace".to_string()],
            }),
            table_border_style: TableBorderStyle::All,
            ..RenderingOptions::default()
        },
        ..BridgeSettingsInput::default()
    });

    assert_eq!(settings["resolvedLocale"], "ja");
    assert_eq!(settings["effectiveInput"]["locale"], "ja");
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["syntaxTheme"],
        "github-dark"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["headingFont"],
        "system-default"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["bodyFont"],
        "system-default"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["codeFont"],
        "system-default"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["tableBorderStyle"],
        "all"
    );
    assert_eq!(settings["controls"]["syntaxTheme"]["enabled"], false);
    assert_eq!(settings["controls"]["headingFont"]["enabled"], false);
    assert_eq!(settings["controls"]["tableBorderStyle"]["enabled"], true);
    assert!(settings["controls"]["syntaxTheme"]["reason"]
        .as_str()
        .is_some_and(|reason| !reason.is_empty()));
    assert!(settings["effectiveInput"]["renderingOptions"]["customHeadingFont"].is_null());
    assert!(settings["effectiveInput"]["renderingOptions"]["customBodyFont"].is_null());
    assert!(settings["effectiveInput"]["renderingOptions"]["customCodeFont"].is_null());
    assert_eq!(
        option_ids(&settings["availableOptions"]["syntaxTheme"]),
        expected_ids(&["github-dark"])
    );
    assert_eq!(
        option_ids(&settings["availableOptions"]["plan"]),
        expected_ids(&["free", "premium"])
    );
    assert_eq!(
        option_ids(&settings["availableOptions"]["tableBorderStyle"]),
        expected_ids(&["none", "inner", "all", "vertical", "horizontal"])
    );
    assert!(settings["availableOptions"]["plan"][0]["label"]
        .as_str()
        .is_some_and(|label| !label.is_empty()));
    assert!(settings["availableOptions"]["locale"]
        .as_array()
        .expect("locale options should be an array")
        .iter()
        .any(|option| option["id"] == "ja"
            && option["label"]
                .as_str()
                .is_some_and(|label| !label.is_empty())));
    assert!(settings["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array")
        .iter()
        .any(|diagnostic| diagnostic["message"] == settings["controls"]["syntaxTheme"]["reason"]));
    assert!(settings["features"]
        .as_array()
        .expect("features should be an array")
        .contains(&serde_json::Value::String("gfm".to_string())));
    assert!(!settings["features"]
        .as_array()
        .expect("features should be an array")
        .contains(&serde_json::Value::String("plantuml".to_string())));
}

#[test]
fn settings_resolver_preserves_premium_look_and_feel_options() {
    let settings = settings_value(BridgeSettingsInput {
        plan: Plan::Premium,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Light,
            syntax_theme: SyntaxTheme::TokyoNight,
            heading_font: HeadingFont::Custom,
            custom_heading_font: Some(FontFamilySpec {
                family: "\"Bridge Heading\"".to_string(),
                fallbacks: vec!["serif".to_string()],
            }),
            body_font: BodyFont::Custom,
            custom_body_font: Some(FontFamilySpec {
                family: "\"Bridge Body\"".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            }),
            code_font: CodeFont::Custom,
            custom_code_font: Some(FontFamilySpec {
                family: "\"Bridge Code\"".to_string(),
                fallbacks: vec!["monospace".to_string()],
            }),
            table_border_style: TableBorderStyle::Vertical,
            ..RenderingOptions::default()
        },
        ..BridgeSettingsInput::default()
    });

    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["syntaxTheme"],
        "tokyo-night"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["headingFont"],
        "custom"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["bodyFont"],
        "custom"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["codeFont"],
        "custom"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["tableBorderStyle"],
        "vertical"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["customHeadingFont"]["family"],
        "\"Bridge Heading\""
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["customBodyFont"]["family"],
        "\"Bridge Body\""
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["customCodeFont"]["family"],
        "\"Bridge Code\""
    );
    assert_eq!(settings["controls"]["syntaxTheme"]["enabled"], true);
    assert!(option_ids(&settings["availableOptions"]["syntaxTheme"])
        .contains(&"tokyo-night".to_string()));
    assert_eq!(
        option_ids(&settings["availableOptions"]["headingFont"]),
        expected_ids(&["system-default", "custom"])
    );
    assert_eq!(
        option_ids(&settings["availableOptions"]["codeFont"]),
        expected_ids(&["system-default", "custom"])
    );
    assert!(settings["availableOptions"]["syntaxTheme"]
        .as_array()
        .expect("syntax theme options should be an array")
        .iter()
        .all(|option| option["id"].as_str().is_some()
            && option["label"]
                .as_str()
                .is_some_and(|label| !label.is_empty())));
    assert!(settings["features"]
        .as_array()
        .expect("features should be an array")
        .contains(&serde_json::Value::String("plantuml".to_string())));
}

#[test]
fn settings_resolver_falls_back_to_english_locale_options() {
    let settings = settings_value(BridgeSettingsInput {
        locale: Some("pt-BR".to_string()),
        plan: Plan::Premium,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Light,
            ..RenderingOptions::default()
        },
        ..BridgeSettingsInput::default()
    });

    assert_eq!(settings["resolvedLocale"], "en");
    assert_eq!(settings["effectiveInput"]["locale"], "en");
    assert_eq!(
        option_ids(&settings["availableOptions"]["locale"]),
        expected_ids(&["en", "es", "fr", "ja", "ko", "zh-Hans", "zh-Hant"])
    );
    assert!(settings["availableOptions"]["locale"]
        .as_array()
        .expect("locale options should be an array")
        .iter()
        .all(|option| option["id"].as_str().is_some()
            && option["label"]
                .as_str()
                .is_some_and(|label| !label.is_empty())));
}

#[test]
fn settings_resolver_preserves_system_default_fonts() {
    let settings = settings_value(BridgeSettingsInput {
        plan: Plan::Premium,
        rendering_options: RenderingOptions {
            color_scheme: ColorScheme::Light,
            heading_font: HeadingFont::SystemDefault,
            body_font: BodyFont::SystemDefault,
            code_font: CodeFont::SystemDefault,
            ..RenderingOptions::default()
        },
        ..BridgeSettingsInput::default()
    });

    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["headingFont"],
        "system-default"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["bodyFont"],
        "system-default"
    );
    assert_eq!(
        settings["effectiveInput"]["renderingOptions"]["codeFont"],
        "system-default"
    );
}

fn settings_value(input: BridgeSettingsInput) -> serde_json::Value {
    serde_json::to_value(resolve_render_settings(input)).expect("settings should serialize")
}

fn option_ids(options: &serde_json::Value) -> Vec<String> {
    options
        .as_array()
        .expect("options should be an array")
        .iter()
        .map(|option| {
            option["id"]
                .as_str()
                .expect("option id should be a string")
                .to_string()
        })
        .collect()
}

fn expected_ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_string()).collect()
}

fn assert_bridge_locked_diagnostics_at_least(diagnostics: &[RenderDiagnostic], count: usize) {
    let actual = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == LOCKED_FEATURE_CODE
                && diagnostic.engine == DiagnosticEngine::Bridge
                && diagnostic.severity == Severity::Warning
        })
        .count();
    assert!(
        actual >= count,
        "expected at least {count} bridge locked-feature diagnostics, got {actual}: {diagnostics:?}"
    );
}

fn has_bridge_locked_diagnostic(diagnostics: &[RenderDiagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LOCKED_FEATURE_CODE && diagnostic.engine == DiagnosticEngine::Bridge
    })
}

fn kind_count(entries: &[KindCount], kind: &str) -> u32 {
    entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.count)
        .unwrap_or(0)
}
