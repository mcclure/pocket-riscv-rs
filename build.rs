use std::path::Path;

/// Put the linker script somewhere the linker can find it.
fn main() {
    let dest_path = Path::new("external/openfpga-litex/lang/linker");

    // "I have no idea why this must be canonical and can't use relative paths" -- agg23
    println!(
        "cargo:rustc-link-search={}",
        dest_path.canonicalize().unwrap().display()
    );

    println!("cargo:rerun-if-changed=regions.ld");
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");

    // GOT SLINT? PUT IT HERE
}
