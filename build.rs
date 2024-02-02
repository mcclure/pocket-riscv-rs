use std::{io::Write, path::Path};

use rgb565::Rgb565;

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
    const FILES: [[&str;2];3] = [
        ["resource/playfield_bg.png", "playfield_bg.bin"],
        ["resource/little witch/player_hit_s_0000.png", "player_hit.bin"],
        ["resource/blobber/blobber_attackbase_0000.png", "blobber_attack.bin"]
    ];
    for [src_name, dst_name] in FILES {
        println!("cargo:rerun-if-changed={}", src_name);
        let im = image::open(Path::new(src_name)).unwrap();
        let im_buf: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = im.into();
        let mut file = std::fs::File::create(Path::new(&std::env::var("OUT_DIR").unwrap()).join(dst_name)).unwrap();
        for (_x, _y, pixel) in im_buf.enumerate_pixels() {
            let image::Rgb([r,g,b]) = *pixel;
            let packed = Rgb565::from_rgb888_components(r, g, b).to_rgb565();
            let pixel = packed.to_le_bytes();
            //pix.extend_from_slice(&pixel);
            file.write_all(&pixel).unwrap();
        }
    } 
}
