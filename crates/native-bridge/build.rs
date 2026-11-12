use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let package = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    craftguild_cli_build::emit(
        package.parent().unwrap().parent().unwrap(),
        "oriha-native-bridge",
    )
    .expect("Run make prepare or tools/artifacts.sh prepare for this native target");
    println!("cargo:rerun-if-env-changed=ORIHA_NATIVE_BRIDGE_GIT_SHORT_HASH");

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set by Cargo"),
    );
    emit_git_rerun_hints(&manifest_dir);

    let git_short_hash = env::var("ORIHA_NATIVE_BRIDGE_GIT_SHORT_HASH")
        .ok()
        .filter(|value| is_valid_git_short_hash(value))
        .or_else(|| git_short_hash(&manifest_dir))
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=ORIHA_NATIVE_BRIDGE_GIT_SHORT_HASH={git_short_hash}");
}

fn git_short_hash(current_dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .current_dir(current_dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    is_valid_git_short_hash(value).then(|| value.to_string())
}

fn emit_git_rerun_hints(current_dir: &Path) {
    let Some(git_dir) = git_dir(current_dir) else {
        return;
    };

    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    println!(
        "cargo:rerun-if-changed={}",
        git_dir.join("packed-refs").display()
    );

    if let Ok(head) = std::fs::read_to_string(git_dir.join("HEAD")) {
        if let Some(reference) = head.strip_prefix("ref: ") {
            println!(
                "cargo:rerun-if-changed={}",
                git_dir.join(reference.trim()).display()
            );
        }
    }
}

fn git_dir(current_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(current_dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let path = PathBuf::from(value);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(current_dir.join(path))
    }
}

fn is_valid_git_short_hash(value: &str) -> bool {
    (7..=40).contains(&value.len()) && value.chars().all(|character| character.is_ascii_hexdigit())
}
