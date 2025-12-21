use std::fs;
use std::path::Path;

fn main() {
    // Read the version from the VERSION file in the workspace root
    let version_path = Path::new("..").join("VERSION");

    println!("cargo:rerun-if-changed={}", version_path.display());

    if let Ok(version) = fs::read_to_string(&version_path) {
        let version = version.trim();
        println!("cargo:rustc-env=RYURI_VERSION={}", version);
    } else {
        // Fallback or warning if VERSION file is missing
        println!("cargo:warning=VERSION file not found, defaulting RYURI_VERSION to 'unknown'");
        println!("cargo:rustc-env=RYURI_VERSION=unknown");
    }
}
