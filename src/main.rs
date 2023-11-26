#![no_std]
#![no_main]
#![allow(unused_parens)]

use core::panic::PanicInfo;
use core::slice::from_raw_parts_mut;

extern crate alloc;

use embedded_alloc::Heap;
use litex_hal as hal;
use litex_pac as pac;
use litex_openfpga::*;
use riscv_rt::entry;
use core::cell::UnsafeCell;

// Definition is required for uart_printer.rs to work
hal::uart! {
    UART: pac::UART,
}

#[repr(u16)]
#[allow(dead_code)]
enum PocketControls {
    DpadUp     = 1<<0,
    DpadDown   = 1<<1,
    DpadLeft   = 1<<2,
    DpadRight  = 1<<3,
    FaceA      = 1<<4,
    FaceB      = 1<<5,
    FaceX      = 1<<6,
    FaceY      = 1<<7,
    TrigL1     = 1<<8,
    TrigR1     = 1<<9,
    TrigL2     = 1<<10,
    TrigR2     = 1<<11,
    TrigL3     = 1<<12,
    TrigR3     = 1<<13,
    FaceSelect = 1<<14,
    FaceStart  = 1<<15,
}

// const TEST_ADDR: *mut u32 = (0xF0001800 + 0x0028) as *mut u32;

// Fix for missing main functions
#[no_mangle]
fn fminf(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

#[no_mangle]
fn fmaxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

use core::mem::MaybeUninit;

const HEAP_SIZE: usize = 200 * 1024;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic:");
    println!("{info}");

    loop {}
}

#[global_allocator]
static HEAP: Heap = Heap::empty();

const DISPLAY_WIDTH: usize = 266;
const DISPLAY_HEIGHT: usize = 240;

const READ_LENGTH: usize = 0x10000;

fn render_init(framebuffer_address: *mut u16) {
    let framebuffer = unsafe { from_raw_parts_mut(framebuffer_address, DISPLAY_WIDTH * DISPLAY_HEIGHT) };

    const PIXEL_MAX:usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
    for idx in 0..PIXEL_MAX {
        framebuffer[idx] = 0;
    }
}

fn pixel(framebuffer_address: *mut u16, x: usize, y: usize) -> &'static mut u16 {
    let framebuffer = unsafe { from_raw_parts_mut(framebuffer_address as *mut u16, READ_LENGTH) };

    &mut framebuffer[y * DISPLAY_WIDTH + x]
}

// This is the entry point for the application.
// It is not allowed to return.
#[entry]
fn main() -> ! {
    let peripherals = unsafe { pac::Peripherals::steal() };

    // Initialize the allocator BEFORE you use it
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) };

    println!("-- ot3 --");

    // Framebuffer pointer
    let fb:*mut u16 = peripherals.VIDEO_FRAMEBUFFER.dma_base.read().bits() as *mut u16;

    render_init(fb);

    // "APP"
    {
        let main_modulus: UnsafeCell<i32> = UnsafeCell::new(254);

        let mut next_byte = {
            let mut n:i32 = 0;
            let mut i:i32 = 1;
            let main_modulus = &main_modulus;
            move || {
                n = n.wrapping_add(i);
                let v:u8 = (n as u32) as u8;
                let modulus = unsafe { *main_modulus.get() };
                if n>modulus || i<=0 {
                    i = i.wrapping_add(1);
                    n = n % modulus;
                }
                v
            }
        };

        fn to_565(byte:u8, color:u8) -> u16 {
            let byte = byte as u16;
            let mut value:u16 = 0;
            let r:u16 = if color & 1 == 0 { 0 } else { 0xFFFF };
            let g:u16 = if color & 2 == 0 { 0 } else { 0xFFFF };
            let b:u16 = if color & 4 == 0 { 0 } else { 0xFFFF };
            value |= ( ((byte^r) & 0b11110000) >> 4);
            value <<= 1;
            value |= ( ((byte^r) & 0b00001000) >> 3);
            value <<= 4;
            value |= ( ((byte^g) & 0b11110000) >> 4);
            value <<= 2;
            value |= ( ((byte^g) & 0b00000110) >> 1);
            value <<= 4;
            value |= ( ((byte^b) & 0b11110000) >> 4);
            value <<= 1;
            value |= ( ((byte^b) & 0b00000001) >> 0);
            value
        }

        // Change sample rate and mulaw
        // Note: An entirely unreasonable amount of the "game logic" occurs here
        let mut next_short = {
            let main_modulus = &main_modulus;
            let mut downsample:u32 = 8;
            let mut down_n: u32 = downsample;
            let mut last:u16 = 0;

            let mut pixel_offset = 0;

            let mut cont1_key_last = 0;

            let mut paused = false;

            let mut hilbert_order:usize = 5;

            let mut clear_want = false;

            let mut color_scheme_raw:u8 = 0;
            let mut color_scheme:u8 = 0;

            move || {
                if down_n >= downsample {
                    // Controls
                    let cont1_key = peripherals.APF_INPUT.cont1_key.read().bits() as u16; // Crop out analog sticks
                    let cont1_key_edge = (!cont1_key_last) & cont1_key;
                    cont1_key_last = cont1_key;

                    use PocketControls::*;

                    // Pause
                    if cont1_key_edge & FaceSelect as u16 != 0 {
                        paused = !paused;
                    }

                    // Size control
                    let order_bump:isize = if cont1_key_edge & DpadLeft as u16 != 0 { -1 }
                        else if cont1_key_edge & DpadRight as u16 != 0 { 1 }
                        else { 0 };

                    if order_bump != 0 {
                        let hilbert_order_new = (hilbert_order as isize + order_bump) as usize;
                        if hilbert_order_new >= 2 && hilbert_order_new <= 8 {
                            hilbert_order = hilbert_order_new;
                            pixel_offset = 0;
                            //if order_bump>0 {
                                clear_want = true;
                            //}
                        }
                    }

                    // Speed control
                    let speed_bump:i32 = if cont1_key_edge & DpadUp as u16 != 0 { -1 }
                        else if cont1_key_edge & DpadDown as u16 != 0 { 1 }
                        else { 0 };

                    if speed_bump != 0 {
                        let downsample_new = (downsample as i32 + speed_bump) as u32;
                        if downsample_new > 0 && downsample_new < 32 {
                            downsample = downsample_new;
                        }
                    }

                    // Algorithm control
                    // Left-right: Slow
                    let modulus_bump:i32 = if cont1_key_edge & FaceY as u16 != 0 { -1 }
                        else if cont1_key_edge & FaceA as u16 != 0 { 1 }
                        else { 0 };

                    if modulus_bump != 0 {
                        let modulus_old = unsafe { *main_modulus.get() };
                        let modulus_new = modulus_old + modulus_bump;
                        if modulus_new > 1 && modulus_new < 1<<17 {
                            unsafe { *main_modulus.get() = modulus_new; };
                        }
                    }

                    // Up-down: Fast
                    let modulus_bump:i32 = if cont1_key_edge & FaceB as u16 != 0 { -256 }
                        else if cont1_key_edge & FaceX as u16 != 0 { 256 }
                        else { 0 };

                    if modulus_bump != 0 {
                        let modulus_old = unsafe { *main_modulus.get() };
                        let modulus_new = modulus_old + modulus_bump;
                        if modulus_new > 1 && modulus_new < 1<<17 {
                            unsafe { *main_modulus.get() = modulus_new; };
                        }
                    }

                    // Color control
                    let color_bump:i8 = if cont1_key_edge & TrigL1 as u16 != 0 { -1 }
                        else if cont1_key_edge & TrigR1 as u16 != 0 { 1 }
                        else { 0 };

                    if color_bump != 0 {
                        let color_new = (color_scheme_raw as i8 + color_bump).rem_euclid(8) as u8;
    //                    if color_new >= 0 && color_new < 8 {
                            color_scheme_raw = color_new as u8;
                            color_scheme = color_scheme_raw;
                            if color_scheme & 4 != 0 { color_scheme ^= 3; }
    //                    }
                    }

                    if !paused { 
                        // Sound
                        let byte = next_byte();
                        last = mulaw_decode(byte);

                        // Draw pixel
                        let hilbert_axis:usize = (1<<hilbert_order);
                        let hilbert_max:usize = hilbert_axis*hilbert_axis;

                        let hilbert_scale:usize = if hilbert_order < 7 { DISPLAY_HEIGHT/hilbert_axis } else { DISPLAY_WIDTH/hilbert_axis }; // FIXME: Assumes screen is wider than tall and "7" is the attractive crossover point 
                        let hilbert_recenter_y:isize = (DISPLAY_HEIGHT as isize-hilbert_axis as isize*hilbert_scale as isize)/2;
                        let hilbert_recenter_x:isize = (DISPLAY_WIDTH as isize-hilbert_axis as isize*hilbert_scale as isize)/2;

                        let color = to_565(byte, color_scheme);
                        let (pixel_x, pixel_y) = hilbert_2d::usize::h2xy_discrete(pixel_offset, hilbert_order, hilbert_2d::Variant::Liu1);

                        if clear_want { // This is kinda expensive :(
                            render_init(fb);
                            clear_want = false;
                        }

                        if hilbert_recenter_y >= 0 { // If square fits within screen, skip safety test. // FIXME: Assumes screen is wider than tall; in future that might not be true.
                            for sub_pixel_y in 0..hilbert_scale {
                                for sub_pixel_x in 0..hilbert_scale {
                                    let true_pixel_x = pixel_x*hilbert_scale + sub_pixel_x + hilbert_recenter_x as usize;
                                    let true_pixel_y = pixel_y*hilbert_scale + sub_pixel_y + hilbert_recenter_y as usize;
                                    *pixel(fb, true_pixel_x, true_pixel_y) = color;
                                }
                            }
                        } else { // Square larger than screen
                            for sub_pixel_y in 0..hilbert_scale {
                                for sub_pixel_x in 0..hilbert_scale {
                                    let true_pixel_x:isize = (pixel_x*hilbert_scale + sub_pixel_x) as isize + hilbert_recenter_x;
                                    let true_pixel_y:isize = (pixel_y*hilbert_scale + sub_pixel_y) as isize + hilbert_recenter_y;
                                    if true_pixel_y>= 0 && true_pixel_y < DISPLAY_HEIGHT as isize {
                                        *pixel(fb, true_pixel_x as usize, true_pixel_y as usize) = color;
                                    }
                                }
                            }
                        }

                        pixel_offset += 1;
                        if pixel_offset >= hilbert_max { pixel_offset = 0; }
                    }
                    down_n = 0;
                }
                down_n += 1;
                last
            }
        };

        // http://neyric.com/2006/10/14/decoding-mu-law-audio-stream-to-pcm/
        fn mulaw_decode(mulaw:u8) -> u16 {
            let mut mulaw = mulaw;
            mulaw = !mulaw;
            let sign:u8 = mulaw & 0x80;
            let exponent:u32 = (mulaw as u32 & 0x70) >> 4;
            let mut data:i32 = (mulaw & 0x0f) as i32;
            data |= 0x10;
            data <<= 1;
            data += 1;
            data <<= exponent + 2;
            data -= 0x84;
            ((if sign == 0 { data } else { -data }) as u32) as u16
        }

        loop {
            let mut buffer_fill = peripherals.APF_AUDIO.buffer_fill.read().bits();

            while buffer_fill > 10 {
                // Busy wait until the buffer is mostly empty
                buffer_fill = peripherals.APF_AUDIO.buffer_fill.read().bits();
            }

            for _ in 0..256 {
                let mut value:u32 = next_short() as u32;
                //value = value / 4;
                value = value | (value << 16);

                peripherals.APF_AUDIO.out.write(|w| unsafe { w.bits(value) })
            }

            unsafe { peripherals.APF_AUDIO.playback_en.write(|w| w.bits(1)) };
        }
    }

    // Unreachable
}
