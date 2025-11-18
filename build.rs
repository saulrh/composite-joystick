fn main() {
    // Re-run the build script if Cargo.toml or patches change
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=patches/");

    // Apply patches to dependencies
    patch_crate::run().expect("Failed to apply dependency patches");
}
