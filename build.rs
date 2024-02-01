use std::path::Path;

use rgb565::Rgb565;

use rkyv::{Archive, Serialize};

#[derive(/*Archive, Deserialize*/, Serialize, Debug, PartialEq)]
#[archive(
    // This will generate a PartialEq impl between our unarchived and archived
    // types:
    compare(PartialEq),
    // bytecheck can be used to validate your data if you want. To use the safe
    // API, you have to derive CheckBytes for the archived type:
    check_bytes,
)]
// Derives can be passed through to the generated type:
#[archive_attr(derive(Debug))]
struct RawImage {
    w:u16, h:u16,
    pixels: Vec<u16>,
}

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
        use litex_pac::constants::{MAX_DISPLAY_WIDTH, MAX_DISPLAY_HEIGHT};
        const MAX_DISPLAY_WIDTH_EXPECTED: u32 = 266;
        const MAX_DISPLAY_HEIGHT_EXPECTED: u32 = 240;
        if MAX_DISPLAY_WIDTH != MAX_DISPLAY_WIDTH_EXPECTED
        || MAX_DISPLAY_HEIGHT != MAX_DISPLAY_HEIGHT_EXPECTED
        {
            println!("cargo:warning=This app was designed for a screen of {MAX_DISPLAY_WIDTH_EXPECTED}x{MAX_DISPLAY_HEIGHT_EXPECTED}. It is being compiled for a screen of {MAX_DISPLAY_WIDTH}x{MAX_DISPLAY_HEIGHT}. Check to make sure it looks good (and if not, edit the \"App Properties\" constants in main.rs), then edit this warning in build.rs.");
        }
    }

    // GOT SLINT? PUT IT HERE

    // Asset loads
    let im = image::open(Path::new("resource/playfield_bg.png")).unwrap();
    let im_buf: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = im.into();
    let mut pix:Vec<u16> = Default::default();
    for (_x, _y, pixel) in im_buf.enumerate_pixels() {
        let image::Rgb([r,g,b]) = *pixel;
        let packed = Rgb565::from_rgb888_components(r, g, b).to_rgb565();
        pix.push(packed);
    }
}
