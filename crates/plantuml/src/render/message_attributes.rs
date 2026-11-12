//! Semantic message attributes and feature markers.
use super::*;

pub(super) fn message_class_attr(message: &MessageLayout) -> String {
    let mut classes = vec!["message".to_owned()];
    let features = &message.arrow_features;

    if features.lost {
        classes.push("lost-message".to_owned());
    }
    if features.thin {
        classes.push("thin-arrow".to_owned());
    }
    if let Some(half) = features.half {
        classes.push("half-arrow".to_owned());
        classes.push(format!("half-arrow-{}", message_arrow_half_name(half)));
    }
    if features.open_circle {
        classes.push("open-circle".to_owned());
    }
    if features.start_circle {
        classes.push("start-open-circle".to_owned());
    }
    if features.end_circle {
        classes.push("end-open-circle".to_owned());
    }
    if features.start_cross {
        classes.push("start-cross".to_owned());
    }
    if features.end_cross {
        classes.push("end-cross".to_owned());
    }
    for (side, head) in [("start", features.start_head), ("end", features.end_head)] {
        if let Some(head) = head {
            classes.push(format!("{side}-arrowhead-{}", arrow_head_name(head)));
        }
    }
    if features.bidirectional || matches!(message.direction, MessageDirection::Bidirectional) {
        classes.push("bidirectional".to_owned());
    }
    if matches!(message.direction, MessageDirection::SelfMessage) {
        classes.push("self-message".to_owned());
    }
    if features.incoming {
        classes.push("incoming-message".to_owned());
    }
    if features.outgoing {
        classes.push("outgoing-message".to_owned());
    }
    if features.unknown_endpoint {
        classes.push("unknown-endpoint".to_owned());
    }
    if features.short_endpoint {
        classes.push("short-endpoint".to_owned());
    }
    if features.parallel {
        classes.push("parallel".to_owned());
    }
    if features.slant.is_some() {
        classes.push("slanted-arrow".to_owned());
    }
    if let Some(shortcut) = &message.activation_shortcut {
        classes.push("activation-shortcut".to_owned());
        classes.push(format!(
            "activation-shortcut-{}",
            activation_kind_name(shortcut.kind)
        ));
        classes.push(format!(
            "activation-target-{}",
            activation_shortcut_target_name(shortcut.target)
        ));
    }

    classes.join(" ")
}

pub(super) fn message_feature_attrs(message: &MessageLayout) -> String {
    let mut attrs = String::new();
    let features = &message.arrow_features;

    push_bool_attr(&mut attrs, "data-lost", features.lost);
    push_bool_attr(&mut attrs, "data-thin", features.thin);
    push_bool_attr(&mut attrs, "data-open-circle", features.open_circle);
    push_bool_attr(&mut attrs, "data-start-circle", features.start_circle);
    push_bool_attr(&mut attrs, "data-end-circle", features.end_circle);
    push_bool_attr(&mut attrs, "data-start-cross", features.start_cross);
    push_bool_attr(&mut attrs, "data-end-cross", features.end_cross);
    push_bool_attr(&mut attrs, "data-bidirectional", features.bidirectional);
    push_bool_attr(&mut attrs, "data-incoming", features.incoming);
    push_bool_attr(&mut attrs, "data-outgoing", features.outgoing);
    push_bool_attr(
        &mut attrs,
        "data-unknown-endpoint",
        features.unknown_endpoint,
    );
    push_bool_attr(&mut attrs, "data-short-endpoint", features.short_endpoint);
    push_bool_attr(&mut attrs, "data-parallel", features.parallel);

    if let Some(half) = features.half {
        write!(
            attrs,
            r#" data-arrow-half="{}""#,
            message_arrow_half_name(half)
        )
        .expect("writing to String cannot fail");
    }
    if let Some(head) = features.start_head {
        write!(
            attrs,
            r#" data-start-arrow-head="{}""#,
            arrow_head_name(head)
        )
        .expect("writing to String cannot fail");
    }
    if let Some(head) = features.end_head {
        write!(attrs, r#" data-end-arrow-head="{}""#, arrow_head_name(head))
            .expect("writing to String cannot fail");
    }
    if let Some(color) = &features.color {
        write!(attrs, r#" data-arrow-color="{}""#, escape_attr(color))
            .expect("writing to String cannot fail");
    }
    if let Some(slant) = features.slant {
        write!(attrs, r#" data-slant="{slant}""#).expect("writing to String cannot fail");
    }
    if let Some(shortcut) = &message.activation_shortcut {
        write!(
            attrs,
            r#" data-activation-shortcut="{}" data-activation-target="{}""#,
            activation_kind_name(shortcut.kind),
            activation_shortcut_target_name(shortcut.target)
        )
        .expect("writing to String cannot fail");
        if let Some(color) = &shortcut.color {
            write!(attrs, r#" data-activation-color="{}""#, escape_attr(color))
                .expect("writing to String cannot fail");
        }
    }
    if !matches!(message.from_endpoint_kind, MessageEndpointKind::Participant) {
        write!(
            attrs,
            r#" data-from-endpoint-kind="{}""#,
            message_endpoint_kind_name(message.from_endpoint_kind)
        )
        .expect("writing to String cannot fail");
    }
    if !matches!(message.to_endpoint_kind, MessageEndpointKind::Participant) {
        write!(
            attrs,
            r#" data-to-endpoint-kind="{}""#,
            message_endpoint_kind_name(message.to_endpoint_kind)
        )
        .expect("writing to String cannot fail");
    }

    attrs
}

pub(super) fn push_bool_attr(attrs: &mut String, name: &str, enabled: bool) {
    if enabled {
        write!(attrs, r#" {name}="true""#).expect("writing to String cannot fail");
    }
}

pub(super) fn render_message_feature_markers(
    svg: &mut String,
    message: &MessageLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
) {
    render_endpoint_marker(
        svg,
        message.start_x,
        message.y,
        message.from_endpoint_kind,
        "from",
        theme,
    );
    render_endpoint_marker(
        svg,
        message.end_x,
        message.y,
        message.to_endpoint_kind,
        "to",
        theme,
    );

    if let Some(shortcut) = &message.activation_shortcut {
        render_message_activation_shortcut(
            svg,
            message,
            shortcut.target,
            shortcut.kind,
            theme,
            style,
        );
    }
}

pub(super) fn render_endpoint_marker(
    svg: &mut String,
    x: f32,
    y: f32,
    kind: MessageEndpointKind,
    side: &str,
    theme: &PlantUmlTheme,
) {
    let kind_name = message_endpoint_kind_name(kind);
    match kind {
        MessageEndpointKind::Participant => {}
        MessageEndpointKind::Incoming | MessageEndpointKind::Outgoing => {
            write!(
                svg,
                r#"<circle class="endpoint-marker endpoint-{} endpoint-{}" data-endpoint-side="{side}" data-endpoint-kind="{kind_name}" cx="{}" cy="{}" r="4" fill="{}" stroke="{}"/>"#,
                kind_name,
                side,
                format_number(x),
                format_number(y),
                escape_attr(&theme.background_color),
                escape_attr(&theme.message_color),
            )
            .expect("writing to String cannot fail");
        }
        MessageEndpointKind::Unknown | MessageEndpointKind::Short => {
            let label = if matches!(kind, MessageEndpointKind::Unknown) {
                "?"
            } else {
                "..."
            };
            write!(
                svg,
                r#"<text class="endpoint-marker endpoint-{} endpoint-{}" data-endpoint-side="{side}" data-endpoint-kind="{kind_name}" x="{}" y="{}" text-anchor="middle" fill="{}">{}</text>"#,
                kind_name,
                side,
                format_number(x),
                format_number(y + 4.0),
                escape_attr(&theme.text_color),
                label,
            )
            .expect("writing to String cannot fail");
        }
    }
}

pub(super) fn render_message_activation_shortcut(
    svg: &mut String,
    message: &MessageLayout,
    target: MessageActivationTarget,
    kind: ActivationKind,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
) {
    let x = match target {
        MessageActivationTarget::From => message.start_x,
        MessageActivationTarget::To => message.end_x,
    };
    let kind_name = activation_kind_name(kind);
    let target_name = activation_shortcut_target_name(target);
    let fill = message
        .activation_shortcut
        .as_ref()
        .and_then(|shortcut| shortcut.color.as_deref())
        .map(svg_color_value)
        .unwrap_or_else(|| Cow::Borrowed(theme.participant_fill.as_str()));
    let corner_radius = style
        .corner_radius
        .svg_px()
        .map(|px| format!(r#" rx="{px}" ry="{px}""#))
        .unwrap_or_default();

    write!(
        svg,
        r#"<g class="message-activation-shortcut activation activation-{kind_name} activation-target-{target_name}" data-event="activation" data-activation-kind="{kind_name}" data-activation-target="{target_name}" data-line="{}">"#,
        message.line,
    )
    .expect("writing to String cannot fail");

    match kind {
        ActivationKind::Activate | ActivationKind::Create => {
            write!(
                svg,
                r#"<rect x="{}" y="{}" width="12" height="20" fill="{}" stroke="{}"{}/>"#,
                format_number(x - 6.0),
                format_number(message.y + 4.0),
                escape_attr(fill.as_ref()),
                escape_attr(&theme.participant_stroke),
                corner_radius,
            )
            .expect("writing to String cannot fail");
        }
        ActivationKind::Deactivate | ActivationKind::Destroy => {
            write!(
                svg,
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2"/>"#,
                format_number(x - 8.0),
                format_number(message.y + 14.0),
                format_number(x + 8.0),
                format_number(message.y + 14.0),
                escape_attr(&theme.participant_stroke),
            )
            .expect("writing to String cannot fail");
        }
    }

    svg.push_str("</g>");
}
