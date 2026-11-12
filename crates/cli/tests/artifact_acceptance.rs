//! Acceptance of the engine-built CLI, independently of Cargo's test binary.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;

#[derive(PartialEq, Eq)]
enum Entry {
    Directory,
    File(Vec<u8>),
    Link(PathBuf),
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, current: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let value = if kind.is_symlink() {
                Entry::Link(fs::read_link(&path).unwrap())
            } else if kind.is_dir() {
                visit(root, &path, entries);
                Entry::Directory
            } else {
                Entry::File(fs::read(&path).unwrap())
            };
            entries.insert(path.strip_prefix(root).unwrap().to_owned(), value);
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

fn command(binary: &Path, runtime: &Path, arguments: &[&str]) -> Command {
    let mut command = Command::new(binary);
    command
        .args(arguments)
        .current_dir(runtime)
        .env_clear()
        .env("PATH", "")
        .env("HOME", runtime.join("home"))
        .env("GIT_DIR", runtime.join("no-git"))
        .env("HOST", "unrelated-host")
        .stdin(Stdio::null());
    command
}

fn run(binary: &Path, runtime: &Path, arguments: &[&str], input: Option<&[u8]>) -> Vec<u8> {
    let mut command = command(binary, runtime, arguments);
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child.stdin.take().unwrap().write_all(input).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    succeeded(&output, arguments);
    assert!(
        output.stderr.is_empty(),
        "{arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn succeeded(output: &Output, arguments: &[&str]) {
    assert!(
        output.status.success(),
        "{arguments:?}: status={}, {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn variable(name: &str, value: impl AsRef<OsStr>) -> OsString {
    let mut argument = OsString::from(name);
    argument.push("=");
    argument.push(value);
    argument
}

#[test]
#[ignore = "make acceptance supplies a verified engine-built artifact and selected profile"]
fn copied_cli_preserves_offline_metadata_rendering_and_installation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let receipt_path = std::env::var_os("ORIHA_ACCEPTANCE_RECEIPT").expect("Run make acceptance");
    let receipt: Value = serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
    let context = &receipt["context"];
    let prepared = &receipt["prepared"];
    let build = &prepared["build"];
    let binary = Path::new(context["output"].as_str().unwrap());
    let expected_sbom = fs::read(root.join(prepared["sbom"].as_str().unwrap())).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let runtime = temporary.path().join("runtime");
    fs::create_dir(&runtime).unwrap();
    fs::create_dir(runtime.join("home")).unwrap();
    let copied = runtime.join("oriha");
    fs::copy(binary, &copied).unwrap();
    fs::write(runtime.join("craftguild.toml"), "invalid configuration [").unwrap();
    let before = snapshot(&runtime);

    let help = run(&copied, &runtime, &[], None);
    assert_eq!(help, run(&copied, &runtime, &["-h"], None));
    assert_eq!(help, run(&copied, &runtime, &["--help"], None));
    let help_text = std::str::from_utf8(&help).unwrap();
    assert!(help_text.is_ascii());
    let affiliation = "craftguild/oriha.rs";
    assert_eq!(help_text.matches(affiliation).count(), 1);
    let lines = help_text.lines().collect::<Vec<_>>();
    let row = lines
        .iter()
        .position(|line| line.trim() == affiliation)
        .unwrap();
    let width = lines[..row].iter().map(|line| line.len()).max().unwrap();
    assert_eq!(
        lines[row],
        format!(
            "{}{}",
            " ".repeat(width.saturating_sub(affiliation.len()) / 2),
            affiliation
        )
    );

    let version = run(&copied, &runtime, &["-V"], None);
    assert_eq!(version, run(&copied, &runtime, &["--version"], None));
    let version_text = std::str::from_utf8(&version).unwrap();
    assert!(version_text.contains(&format!("oriha {}", build["version"].as_str().unwrap())));
    for field in ["commit", "compiler", "target", "profile"] {
        assert!(
            version_text.contains(build[field].as_str().unwrap()),
            "Version lacks {field}"
        );
    }
    assert_eq!(version_text.contains(" (dirty)"), build["dirty"] == "dirty");
    assert!(!help.contains(&0x1b) && !version.contains(&0x1b));
    assert!(
        run(&copied, &runtime, &["sbom"], None) == expected_sbom,
        "Embedded SBOM bytes differ"
    );
    let sbom: Value = serde_json::from_slice(&expected_sbom).unwrap();
    assert_eq!(sbom["specVersion"], "1.5");
    for subcommand in ["render", "render-json", "inspect", "sbom"] {
        let output = run(&copied, &runtime, &[subcommand, "--help"], None);
        assert!(!std::str::from_utf8(&output).unwrap().contains(affiliation));
    }
    let rendered: Value = serde_json::from_slice(&run(
        &copied,
        &runtime,
        &["render-json", "-"],
        Some(b"# Offline\n"),
    ))
    .unwrap();
    assert!(rendered["html"].as_str().unwrap().contains("<h1"));
    assert!(rendered["html"].as_str().unwrap().contains("Offline"));
    let inspected: Value = serde_json::from_slice(&run(
        &copied,
        &runtime,
        &["inspect", "-"],
        Some(b"# Offline\n"),
    ))
    .unwrap();
    assert!(inspected.is_object());
    assert!(
        snapshot(&runtime) == before,
        "Offline CLI commands changed runtime files"
    );

    let stage = temporary.path().join("stage");
    let mut install = Command::new("make");
    install
        .current_dir(&root)
        .arg("install")
        .arg(variable("DESTDIR", &stage))
        .arg("PREFIX=/opt/craftguild")
        .arg(variable(
            "CARGO_TARGET_DIR",
            context["target_dir"].as_str().unwrap(),
        ))
        .arg(variable("PROFILE", context["profile"].as_str().unwrap()))
        .arg(variable(
            "VARIANT",
            context["variant"].as_str().unwrap_or(""),
        ));
    if context["explicit_target"] == true {
        install.arg(variable("TARGET", context["target"].as_str().unwrap()));
    }
    let installed = install.output().unwrap();
    succeeded(&installed, &["make", "install"]);
    assert!(
        fs::read(stage.join("opt/craftguild/bin/oriha")).unwrap() == fs::read(binary).unwrap(),
        "Installed bytes differ from the built artifact"
    );
}
