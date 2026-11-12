fn main() {
    let package = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    craftguild_cli_build::emit(package.parent().unwrap().parent().unwrap(), "oriha-wasm")
        .expect("Run make prepare or tools/artifacts.sh prepare for this Wasm target");
}
