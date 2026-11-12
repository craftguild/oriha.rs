//! Root presentation precedes product parsing and needs only embedded data.
use clap::CommandFactory;
use craftguild_cli::{help, sbom, version, Brand};
use std::{ffi::OsString, io};

// The shared generator also exposes optional machine metadata for other callers.
#[allow(dead_code)]
mod identity {
    include!(concat!(env!("OUT_DIR"), "/craftguild.rs"));
}
use identity::{BUILD_INFO, SBOM};

const BRAND: Brand<'static> = Brand {
    banner: include_str!("../assets/banner.txt"),
    project: "oriha.rs",
};

pub fn early(args: &[OsString]) -> Result<bool, String> {
    match args {
        [] => root_help()?,
        [flag] if flag == "-h" || flag == "--help" => root_help()?,
        [flag] if flag == "-V" || flag == "--version" => {
            version(&mut io::stdout().lock(), BRAND, BUILD_INFO)
                .map_err(|error| format!("failed to write version: {error}"))?;
        }
        [command] if command == "sbom" => write_sbom()?,
        _ => return Ok(false),
    }
    Ok(true)
}

pub fn root_help() -> Result<(), String> {
    let mut command = super::Cli::command().color(clap::ColorChoice::Never);
    let text = format!("{}\n", command.render_long_help());
    help(&mut io::stdout().lock(), BRAND, &text)
        .map_err(|error| format!("failed to write help: {error}"))
}

pub fn write_sbom() -> Result<(), String> {
    sbom(&mut io::stdout().lock(), SBOM).map_err(|error| format!("failed to write SBOM: {error}"))
}
