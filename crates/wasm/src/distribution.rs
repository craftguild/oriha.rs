//! Keep the verified library SBOM reachable without changing typed binding metadata.
const SBOM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/craftguild-sbom.json"));

/// Return the embedded dependency SBOM bytes; read its length separately.
#[no_mangle]
pub extern "C" fn oriha_wasm_sbom_bytes() -> *const u8 {
    SBOM.as_ptr()
}

/// Return the embedded dependency SBOM byte length.
#[no_mangle]
pub extern "C" fn oriha_wasm_sbom_size() -> usize {
    SBOM.len()
}
