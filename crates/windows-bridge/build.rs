fn main() {
    let package = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    craftguild_cli_build::emit(
        package.parent().unwrap().parent().unwrap(),
        "oriha-windows-bridge",
    )
    .expect("Run make prepare or tools/artifacts.sh prepare for this Windows target");
    cxx_build::bridge("src/lib.rs").compile("oriha-windows-bridge-cxx");

    println!("cargo:rerun-if-changed=src/lib.rs");
}
