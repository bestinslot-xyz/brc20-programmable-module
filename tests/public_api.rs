// Run with `INSTA_UPDATE=always cargo test` to update the public API snapshot

/// Nightly used to build rustdoc JSON. public_api::MINIMUM_NIGHTLY_RUST_VERSION predates the
/// rust-version some dependencies require, so a newer nightly whose JSON public-api still reads
/// is pinned here.
const RUSTDOC_NIGHTLY: &str = "nightly-2026-10-01";

#[test]
fn public_api() {
    // Install the nightly toolchain if it is missing
    rustup_toolchain::install(RUSTDOC_NIGHTLY).unwrap();

    // Build rustdoc JSON
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(RUSTDOC_NIGHTLY)
        .all_features(true)
        .build()
        .unwrap();

    // Derive the public API from the rustdoc JSON
    let public_api = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .omit_auto_derived_impls(true)
        .omit_auto_trait_impls(true)
        .omit_blanket_impls(true)
        .build()
        .unwrap();

    // Assert that the public API looks correct
    insta::assert_snapshot!(public_api);
}
