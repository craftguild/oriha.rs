//! Product commands retain their existing renderer and JSON contracts.
mod presentation;

use clap::{Args, Parser, Subcommand, ValueEnum};
use oriha_core::{
    default_watermark_options, render, render_json, AppearanceOptions, ColorScheme, FeatureFlags,
    FontFamilySpec, PrintOptions, RenderInput, RenderMode, RenderOptions, TableBorderStyle,
};
use std::{fs, io::Read, process};

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result =
        presentation::early(&args).and_then(
            |handled| {
                if handled {
                    Ok(())
                } else {
                    run(Cli::parse())
                }
            },
        );
    if let Err(error) = result {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let Some(command) = cli.command else {
        return presentation::root_help();
    };

    match command {
        Command::Sbom => presentation::write_sbom()?,
        Command::Render(options) => {
            let markdown = read_markdown(options.path.clone())?;
            let output = render(RenderInput {
                markdown,
                mode: if options.plain {
                    RenderMode::SemanticHtml
                } else {
                    RenderMode::StyledHtml
                },
                appearance: options.appearance_options(),
                render_options: options.render_options(),
                features: options.feature_flags(),
                ..default_input()
            });
            print_rendered_html(output);
        }
        Command::RenderJson { path } => {
            let markdown = read_markdown(path)?;
            let input = RenderInput {
                markdown,
                mode: RenderMode::StyledHtml,
                ..default_input()
            };
            let input_json = serde_json::to_string(&input)
                .map_err(|error| format!("failed to serialize render input: {error}"))?;
            println!("{}", render_json(&input_json));
        }
        Command::Inspect { path } => {
            let markdown = read_markdown(path)?;
            let output = render(RenderInput {
                markdown,
                mode: RenderMode::SemanticHtml,
                ..default_input()
            });
            let json = serde_json::to_string_pretty(&output)
                .map_err(|error| format!("failed to serialize render output: {error}"))?;
            println!("{json}");
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "oriha", about = "oriha.rs CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the embedded distribution SBOM as CycloneDX JSON.
    Sbom,
    /// Render Markdown to HTML.
    Render(Box<RenderCommand>),
    /// Render Markdown to JSON output.
    #[command(name = "render-json")]
    RenderJson {
        /// Markdown input path. Use - or omit to read stdin.
        path: Option<String>,
    },
    /// Pretty-print RenderOutput JSON.
    Inspect {
        /// Markdown input path. Use - or omit to read stdin.
        path: Option<String>,
    },
}

#[derive(Debug, Args)]
struct RenderCommand {
    /// Output plain semantic HTML without generated CSS.
    #[arg(long)]
    plain: bool,
    /// Select appearance color scheme.
    #[arg(long, value_enum, default_value_t = CliColorScheme::Light)]
    color_scheme: CliColorScheme,
    /// Select appearance style.
    #[arg(long, default_value = "rich")]
    style: String,
    /// Select syntax token palette.
    #[arg(long, default_value = "auto")]
    syntax_theme: String,
    /// Override body font CSS stack.
    #[arg(long)]
    font_family: Option<String>,
    /// Override body font size in CSS pixels.
    #[arg(long)]
    body_font_size_px: Option<f32>,
    /// Override code font CSS stack.
    #[arg(long)]
    code_font_family: Option<String>,
    /// Select table border style.
    #[arg(long, value_enum, default_value_t = CliTableBorderStyle::Inner)]
    table_border_style: CliTableBorderStyle,
    /// Disable rounded corners on rendered blocks.
    #[arg(long)]
    no_rounded_corners: bool,
    /// Disable shadows on rendered blocks.
    #[arg(long)]
    no_shadows: bool,
    /// Expand table cells for PDF/export-friendly full data display.
    #[arg(long)]
    full_table_data: bool,
    /// Add borders around block elements.
    #[arg(long)]
    block_borders: bool,
    /// Enable print CSS.
    #[arg(long)]
    print: bool,
    /// Set @page size when print CSS is enabled.
    #[arg(long)]
    page_size: Option<String>,
    /// Set @page margin when print CSS is enabled.
    #[arg(long)]
    margin: Option<String>,
    /// Render a default watermark.
    #[arg(long)]
    watermark: bool,
    /// Render watermark text.
    #[arg(long)]
    watermark_text: Option<String>,
    /// Render watermark image source.
    #[arg(long)]
    watermark_image: Option<String>,
    /// Override watermark opacity.
    #[arg(long)]
    watermark_opacity: Option<f32>,
    /// Render LaTeX math blocks and inline math as SVG.
    #[arg(long)]
    math: bool,
    /// Render Mermaid fenced code blocks as SVG.
    #[arg(long)]
    mermaid: bool,
    /// Render PlantUML fenced code blocks as SVG.
    #[arg(long)]
    plantuml: bool,
    /// Render all supported diagram fenced code blocks as SVG.
    #[arg(long)]
    diagrams: bool,
    /// Markdown input path. Use - or omit to read stdin.
    path: Option<String>,
}

impl RenderCommand {
    fn appearance_options(&self) -> AppearanceOptions {
        let mut appearance = AppearanceOptions {
            color_scheme: self.color_scheme.into(),
            style: self.style.clone(),
            syntax_theme: self.syntax_theme.clone(),
            table_border_style: self.table_border_style.into(),
            rounded_corners: !self.no_rounded_corners,
            shadows: !self.no_shadows,
            full_table_data: self.full_table_data,
            block_borders: self.block_borders,
            ..AppearanceOptions::default()
        };
        if let Some(font_family) = &self.font_family {
            appearance.body_font_family = FontFamilySpec::from_css_stack(font_family.clone());
        }
        appearance.body_font_size_px = self.body_font_size_px;
        if let Some(code_font_family) = &self.code_font_family {
            appearance.code_font_family = FontFamilySpec::from_css_stack(code_font_family.clone());
        }
        if self.print || self.page_size.is_some() || self.margin.is_some() {
            appearance.print = Some(PrintOptions {
                page_size: self.page_size.clone(),
                margin: self.margin.clone(),
            });
        }
        appearance
    }

    fn render_options(&self) -> RenderOptions {
        let watermark = if self.watermark
            || self.watermark_text.is_some()
            || self.watermark_image.is_some()
            || self.watermark_opacity.is_some()
        {
            let mut watermark = default_watermark_options();
            if let Some(text) = self.watermark_text.clone() {
                watermark.text = text;
            }
            if self.watermark_image.is_some() {
                watermark.image_src = self.watermark_image.clone();
            }
            watermark.opacity = self.watermark_opacity;
            Some(watermark)
        } else {
            None
        };

        RenderOptions {
            watermark,
            ..RenderOptions::default()
        }
    }

    fn feature_flags(&self) -> FeatureFlags {
        FeatureFlags {
            math: self.math,
            mermaid: self.mermaid || self.diagrams,
            plantuml: self.plantuml || self.diagrams,
            ..FeatureFlags::default()
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum CliColorScheme {
    Light,
    Dark,
}

impl From<CliColorScheme> for ColorScheme {
    fn from(value: CliColorScheme) -> Self {
        match value {
            CliColorScheme::Light => Self::Light,
            CliColorScheme::Dark => Self::Dark,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum CliTableBorderStyle {
    None,
    Inner,
    All,
    Vertical,
    Horizontal,
}

impl From<CliTableBorderStyle> for TableBorderStyle {
    fn from(value: CliTableBorderStyle) -> Self {
        match value {
            CliTableBorderStyle::None => Self::None,
            CliTableBorderStyle::Inner => Self::Inner,
            CliTableBorderStyle::All => Self::All,
            CliTableBorderStyle::Vertical => Self::Vertical,
            CliTableBorderStyle::Horizontal => Self::Horizontal,
        }
    }
}

fn print_rendered_html(output: oriha_core::RenderOutput) {
    if let Some(css) = output.css {
        println!("<style>\n{css}\n</style>\n{}", output.html);
    } else {
        println!("{}", output.html);
    }
}

fn read_markdown(path: Option<String>) -> Result<String, String> {
    match path {
        Some(path) if path != "-" => {
            fs::read_to_string(&path).map_err(|error| format!("failed to read {path}: {error}"))
        }
        _ => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|error| format!("failed to read stdin: {error}"))?;
            Ok(buffer)
        }
    }
}

fn default_input() -> RenderInput {
    RenderInput {
        markdown: String::new(),
        locale: None,
        appearance: Default::default(),
        render_options: Default::default(),
        mode: Default::default(),
        features: Default::default(),
        viewport: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_render_args(args: &[&str]) -> RenderCommand {
        let mut argv = vec!["oriha", "render"];
        argv.extend_from_slice(args);
        match Cli::try_parse_from(argv).expect("valid args").command {
            Some(Command::Render(options)) => *options,
            _ => panic!("expected render command"),
        }
    }

    #[test]
    fn render_args_default_to_decorated_output() {
        let options = parse_render_args(&[]);
        let appearance = options.appearance_options();
        let render_options = options.render_options();

        assert!(!options.plain);
        assert!(options.path.is_none());
        assert_eq!(appearance.color_scheme, ColorScheme::Light);
        assert!(appearance.rounded_corners);
        assert!(appearance.shadows);
        assert!(!appearance.full_table_data);
        assert!(!appearance.block_borders);
        assert!(render_options.watermark.is_none());
        assert_eq!(options.feature_flags(), FeatureFlags::default());
    }

    #[test]
    fn render_args_plain_requests_plain_output() {
        let options = parse_render_args(&["--plain", "-"]);

        assert!(options.plain);
        assert_eq!(options.path.as_deref(), Some("-"));
    }

    #[test]
    fn render_args_parse_theme_print_and_watermark_options() {
        let options = parse_render_args(&[
            "--color-scheme=dark",
            "--style",
            "simple",
            "--syntax-theme",
            "monokai",
            "--font-family",
            "\"Noto Sans\", sans-serif",
            "--body-font-size-px",
            "20",
            "--code-font-family",
            "Fira Code, monospace",
            "--table-border-style",
            "all",
            "--no-rounded-corners",
            "--no-shadows",
            "--full-table-data",
            "--block-borders",
            "--print",
            "--page-size",
            "A4 landscape",
            "--margin=10mm",
            "--watermark-text",
            "Draft",
            "--watermark-image",
            "data:image/png;base64,abc",
            "--watermark-opacity",
            "0.35",
            "examples/markdown/comprehensive.md",
        ]);
        let appearance = options.appearance_options();
        let render_options = options.render_options();

        assert_eq!(
            options.path.as_deref(),
            Some("examples/markdown/comprehensive.md")
        );
        assert_eq!(appearance.color_scheme, ColorScheme::Dark);
        assert_eq!(appearance.style, "simple");
        assert_eq!(appearance.syntax_theme, "monokai");
        assert_eq!(appearance.body_font_size_px, Some(20.0));
        assert_eq!(appearance.table_border_style, TableBorderStyle::All);
        assert!(!appearance.rounded_corners);
        assert!(!appearance.shadows);
        assert!(appearance.full_table_data);
        assert!(appearance.block_borders);
        let print = appearance.print.expect("print options");
        assert_eq!(print.page_size.as_deref(), Some("A4 landscape"));
        assert_eq!(print.margin.as_deref(), Some("10mm"));
        let watermark = render_options.watermark.expect("watermark");
        assert_eq!(watermark.text, "Draft");
        assert_eq!(
            watermark.image_src.as_deref(),
            Some("data:image/png;base64,abc")
        );
        assert_eq!(watermark.opacity, Some(0.35));
    }

    #[test]
    fn render_args_reject_invalid_option_values() {
        assert!(Cli::try_parse_from(["oriha", "render", "--color-scheme", "sepia"]).is_err());
        assert!(Cli::try_parse_from(["oriha", "render", "--table-border-style=outer"]).is_err());
        assert!(Cli::try_parse_from(["oriha", "render", "--watermark-opacity", "opaque"]).is_err());
    }

    #[test]
    fn render_args_parse_feature_flags() {
        let math = parse_render_args(&["--math"]);
        assert!(math.feature_flags().math);
        assert!(!math.feature_flags().mermaid);
        assert!(!math.feature_flags().plantuml);

        let diagrams = parse_render_args(&["--diagrams"]);
        assert!(!diagrams.feature_flags().math);
        assert!(diagrams.feature_flags().mermaid);
        assert!(diagrams.feature_flags().plantuml);

        let all = parse_render_args(&["--math", "--mermaid", "--plantuml"]);
        assert!(all.feature_flags().math);
        assert!(all.feature_flags().mermaid);
        assert!(all.feature_flags().plantuml);
    }
}
