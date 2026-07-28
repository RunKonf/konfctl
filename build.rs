fn main() {
    // Allow overriding the version at build time (used by CI for date+sha releases).
    // Usage: KONF_VERSION="2026.04.05-abc1234" cargo build --release
    if let Ok(v) = std::env::var("KONF_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={v}");
    }
    println!("cargo:rerun-if-env-changed=KONF_VERSION");
}
