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

    {
        // Print a human-readable warning if the screen size is surprising.
        use litex_pac::constants::{VIDEO_FRAMEBUFFER_HRES, VIDEO_FRAMEBUFFER_VRES};
        const VIDEO_FRAMEBUFFER_HRES_EXPECTED: u32 = 266;
        const VIDEO_FRAMEBUFFER_VRES_EXPECTED: u32 = 240;
        if VIDEO_FRAMEBUFFER_HRES != VIDEO_FRAMEBUFFER_HRES_EXPECTED
            || VIDEO_FRAMEBUFFER_VRES != VIDEO_FRAMEBUFFER_VRES_EXPECTED
        {
            println!("cargo:warning=This app was designed for a screen of {VIDEO_FRAMEBUFFER_HRES_EXPECTED}x{VIDEO_FRAMEBUFFER_VRES_EXPECTED}. It is being compiled for a screen of {VIDEO_FRAMEBUFFER_HRES}x{VIDEO_FRAMEBUFFER_VRES}. Check to make sure it looks good (and if not, edit the \"App Properties\" constants in main.rs), then edit this warning in build.rs.");
        }
    }

    // GOT SLINT? PUT IT HERE
}
