mod distribution;
#[cxx::bridge(namespace = "oriha::windows_bridge")]
mod ffi {
    extern "Rust" {
        fn native_bridge_version() -> String;
        fn native_bridge_git_short_hash() -> String;
    }
}

pub fn native_bridge_version() -> String {
    oriha_native_bridge::native_bridge_version_info().version
}

pub fn native_bridge_git_short_hash() -> String {
    oriha_native_bridge::native_bridge_version_info().git_short_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_native_bridge_version_parts() {
        assert!(!native_bridge_version().is_empty());
        assert!(!native_bridge_git_short_hash().is_empty());
    }
}
