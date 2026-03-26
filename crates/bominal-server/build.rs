//! Build script for bominal-server.
//!
//! Resolves the OpenSSL/BoringSSL symbol conflict when both `openssl-sys`
//! (from webauthn-rs) and `boring-sys2` (from wreq) are linked into the
//! same binary.
//!
//! The approach: add linker arguments that directly reference the system
//! OpenSSL dynamic libraries, bypassing the static BoringSSL copies that
//! boring-sys2 places on the search path.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "macos" {
        // Homebrew OpenSSL 3 location.
        let brew_prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew/opt/openssl@3"
        } else {
            "/usr/local/opt/openssl@3"
        };

        let lib_dir = format!("{brew_prefix}/lib");
        if std::path::Path::new(&lib_dir).exists() {
            // Force-link the system OpenSSL dylibs by absolute path so the
            // linker uses them instead of boring-sys2's static archives.
            println!("cargo:rustc-link-arg=-Wl,-force_load,{lib_dir}/libcrypto.dylib");
        }
    } else if target_os == "linux" {
        // On Linux (deployment target), the system provides OpenSSL 3.
        // The linker typically resolves correctly via pkg-config, but if
        // boring-sys2's static libs shadow the system copies, force the
        // dynamic versions.
        println!("cargo:rustc-link-arg=-Wl,--push-state,--no-as-needed");
        println!("cargo:rustc-link-arg=-lcrypto");
        println!("cargo:rustc-link-arg=-Wl,--pop-state");
    }
}
