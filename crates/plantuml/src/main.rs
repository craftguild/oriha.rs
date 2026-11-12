use clap::{Parser, ValueEnum};
use oriha_plantuml::{
    DiagramArrowheadStyle, DiagramCornerRadius, DiagramStyle, PlantUml, PlantUmlOptions,
};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process,
};

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("{error}");
        process::exit(1);
    }
}

#[derive(Debug, Parser)]
#[command(name = "oriha-plantuml", about = "Render PlantUML diagrams to SVG.")]
struct Cli {
    /// PlantUML input path. Use - or omit to read stdin.
    #[arg(value_name = "input.puml", default_value = "-")]
    input: String,
    /// SVG output path. Omit to write SVG to stdout.
    #[arg(long, short, value_name = "file.svg")]
    output: Option<PathBuf>,
    /// Participant and block corner radius.
    #[arg(long, value_enum)]
    round: Option<CliCornerRadius>,
    /// Message arrowhead shape.
    #[arg(long, value_enum)]
    arrowhead: Option<CliArrowheadStyle>,
}

impl Cli {
    fn options(&self) -> PlantUmlOptions {
        PlantUmlOptions {
            style: DiagramStyle {
                corner_radius: self.round.map(Into::into),
                arrowhead: self.arrowhead.map(Into::into),
                ..DiagramStyle::default()
            },
            ..PlantUmlOptions::default()
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum CliCornerRadius {
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl From<CliCornerRadius> for DiagramCornerRadius {
    fn from(value: CliCornerRadius) -> Self {
        match value {
            CliCornerRadius::None => Self::None,
            CliCornerRadius::Xs => Self::Xs,
            CliCornerRadius::Sm => Self::Sm,
            CliCornerRadius::Md => Self::Md,
            CliCornerRadius::Lg => Self::Lg,
            CliCornerRadius::Xl => Self::Xl,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum CliArrowheadStyle {
    Triangle,
    Chevron,
    Kite,
}

impl From<CliArrowheadStyle> for DiagramArrowheadStyle {
    fn from(value: CliArrowheadStyle) -> Self {
        match value {
            CliArrowheadStyle::Triangle => Self::Triangle,
            CliArrowheadStyle::Chevron => Self::Chevron,
            CliArrowheadStyle::Kite => Self::Kite,
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    run_with_io(cli, &mut stdin, &mut stdout)
}

fn run_with_io<R, W>(cli: Cli, stdin: &mut R, stdout: &mut W) -> Result<(), String>
where
    R: Read,
    W: Write,
{
    let source = read_input(&cli.input, stdin)?;
    let options = cli.options();
    let rendered = PlantUml::new()
        .render_with_options(&source, &options)
        .map_err(|error| error.to_string())?;

    write_output(cli.output.as_deref(), rendered.svg.as_bytes(), stdout)
}

fn read_input<R>(input: &str, stdin: &mut R) -> Result<String, String>
where
    R: Read,
{
    if input == "-" {
        let mut source = String::new();
        stdin
            .read_to_string(&mut source)
            .map_err(|error| format!("failed to read stdin: {error}"))?;
        return Ok(source);
    }

    fs::read_to_string(input).map_err(|error| format!("failed to read {input}: {error}"))
}

fn write_output<W>(output: Option<&Path>, svg: &[u8], stdout: &mut W) -> Result<(), String>
where
    W: Write,
{
    if let Some(path) = output {
        fs::write(path, svg)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
        return Ok(());
    }

    stdout
        .write_all(svg)
        .map_err(|error| format!("failed to write stdout: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SOURCE: &str = "@startuml\nAlice -> Bob : Hello\n@enduml\n";

    #[test]
    fn args_default_to_stdin_and_stdout() {
        let cli = Cli::parse_from(["oriha-plantuml"]);

        assert_eq!(cli.input, "-");
        assert!(cli.output.is_none());
    }

    #[test]
    fn args_parse_input_and_output() {
        let cli = Cli::parse_from(["oriha-plantuml", "diagram.puml", "--output", "diagram.svg"]);

        assert_eq!(cli.input, "diagram.puml");
        assert_eq!(cli.output, Some(PathBuf::from("diagram.svg")));
    }

    #[test]
    fn args_parse_round_and_arrowhead() {
        let cli = Cli::parse_from(["oriha-plantuml", "--round", "md", "--arrowhead", "kite"]);

        assert_eq!(cli.round, Some(CliCornerRadius::Md));
        assert_eq!(cli.arrowhead, Some(CliArrowheadStyle::Kite));
    }

    #[test]
    fn dash_input_reads_stdin_and_writes_stdout() {
        let cli = Cli::parse_from(["oriha-plantuml", "-"]);
        let mut stdin = SOURCE.as_bytes();
        let mut stdout = Vec::new();

        run_with_io(cli, &mut stdin, &mut stdout).expect("render from stdin");

        let svg = String::from_utf8(stdout).expect("utf8 svg");
        assert!(svg.contains("data-diagram-type=\"SEQUENCE\""));
        assert!(svg.contains("Hello"));
    }

    #[test]
    fn style_options_affect_svg_output() {
        let cli = Cli::parse_from(["oriha-plantuml", "--round", "md", "--arrowhead", "kite"]);
        let mut stdin = SOURCE.as_bytes();
        let mut stdout = Vec::new();

        run_with_io(cli, &mut stdin, &mut stdout).expect("render styled svg");

        let svg = String::from_utf8(stdout).expect("utf8 svg");
        assert!(svg.contains("rx=\"6\""));
        assert!(svg.contains("class=\"arrowhead arrowhead--kite\""));
    }

    #[test]
    fn file_input_writes_output_file() {
        let input = temp_path("input.puml");
        let output = temp_path("output.svg");
        fs::write(&input, SOURCE).expect("write input");

        let cli = Cli::parse_from([
            "oriha-plantuml",
            input.to_str().expect("input path"),
            "--output",
            output.to_str().expect("output path"),
        ]);
        let mut stdin = "".as_bytes();
        let mut stdout = Vec::new();

        run_with_io(cli, &mut stdin, &mut stdout).expect("render from file");

        assert!(stdout.is_empty());
        let svg = fs::read_to_string(&output).expect("read output");
        assert!(svg.contains("data-diagram-type=\"SEQUENCE\""));
        assert!(svg.contains("Hello"));

        let _ = fs::remove_file(input);
        let _ = fs::remove_file(output);
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        std::env::temp_dir().join(format!("oriha-plantuml-{}-{nanos}-{name}", process::id()))
    }
}
