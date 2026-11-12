//! Mermaid theme defaults and JavaScript configuration conversion.
use super::*;

pub(super) fn resolve_body_font_size_px(value: Option<f32>) -> f32 {
    value
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(MermaidTheme::default().font_size)
}

impl Default for MermaidTheme {
    fn default() -> Self {
        Self {
            dark_mode: false,
            font_family: r#""trebuchet ms", verdana, arial, sans-serif"#.to_owned(),
            font_size: 16.0,
            background: None,
            primary_color: None,
            primary_text_color: None,
            primary_border_color: None,
            line_color: None,
            text_color: None,
            main_bkg: None,
            node_bkg: None,
            node_border: None,
            cluster_bkg: None,
            cluster_border: None,
            edge_label_background: None,
        }
    }
}

impl MermaidTheme {
    pub fn to_mermaid_config_value(&self) -> Value {
        let font_size = finite_positive_f32(self.font_size, 16.0);
        let mut root = Map::new();
        root.insert("theme".to_owned(), Value::String("base".to_owned()));
        root.insert("darkMode".to_owned(), Value::Bool(self.dark_mode));
        root.insert(
            "fontFamily".to_owned(),
            Value::String(self.font_family.clone()),
        );
        root.insert("fontSize".to_owned(), json_number_f32(font_size));

        let mut theme_variables = Map::new();
        theme_variables.insert(
            "fontFamily".to_owned(),
            Value::String(self.font_family.clone()),
        );
        theme_variables.insert(
            "fontSize".to_owned(),
            Value::String(format_css_px(font_size)),
        );
        insert_optional_string(
            &mut theme_variables,
            "background",
            self.background.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "primaryColor",
            self.primary_color.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "primaryTextColor",
            self.primary_text_color.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "primaryBorderColor",
            self.primary_border_color.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "lineColor",
            self.line_color.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "textColor",
            self.text_color.as_deref(),
        );
        insert_optional_string(&mut theme_variables, "mainBkg", self.main_bkg.as_deref());
        insert_optional_string(&mut theme_variables, "nodeBkg", self.node_bkg.as_deref());
        insert_optional_string(
            &mut theme_variables,
            "nodeBorder",
            self.node_border.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "clusterBkg",
            self.cluster_bkg.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "clusterBorder",
            self.cluster_border.as_deref(),
        );
        insert_optional_string(
            &mut theme_variables,
            "edgeLabelBackground",
            self.edge_label_background.as_deref(),
        );
        root.insert("themeVariables".to_owned(), Value::Object(theme_variables));

        Value::Object(root)
    }
}
