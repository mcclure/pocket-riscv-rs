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


// Basic platform support

// Definition is required for uart_printer.rs to work
hal::uart! {
    UART: pac::UART,
}

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

// Drawing support

const DISPLAY_WIDTH: usize = pac::constants::MAX_DISPLAY_WIDTH as usize;
const DISPLAY_HEIGHT: usize = pac::constants::MAX_DISPLAY_HEIGHT as usize;

const READ_LENGTH: usize = 0x10000;

fn render_init(framebuffer_address: *mut u16) {
    let framebuffer = unsafe { from_raw_parts_mut(framebuffer_address, DISPLAY_WIDTH * DISPLAY_HEIGHT) };

    const PIXEL_MAX:usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
    for idx in 0..PIXEL_MAX {
        framebuffer[idx] = 0xFFFF;
    }
}

fn pixel(framebuffer_address: *mut u16, x: usize, y: usize) -> &'static mut u16 {
    let framebuffer = unsafe { from_raw_parts_mut(framebuffer_address as *mut u16, READ_LENGTH) };

    &mut framebuffer[y * DISPLAY_WIDTH + x]
}

// Gamepad controls

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

// This is the entry point for the application.
// It is not allowed to return.
#[entry]
fn main() -> ! {
    let peripherals = unsafe { pac::Peripherals::steal() };

    // Initialize the allocator BEFORE you use it
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) };

    println!("-- Strobe --");

    // Framebuffer pointer
    // TODO: Either assert heap and framebuffer are not overlapping, or set VIDEO_FRAMEBUFFER manually
    let fb:*mut u16 = peripherals.VIDEO_FRAMEBUFFER.dma_base.read().bits() as *mut u16;

    render_init(fb);

    // "APP"
    {
        let mut wave:u16 = 0;
        const FREQ_DELTA:u16 = 150; 
        const AUDIO_TARGET:i32 = 48000/60 + 200; // Try to fill audio buffer to this point
        let mut frame_counter:u8 = 0;

        let mut paused = false;
        let mut cont1_key_last = 0;

        cfg_if::cfg_if! {
            if #[cfg(feature = "speed-debug")] { // State used to detect deadline misses
                const SPEED_DEBUG_RATE:u32 = 1; // Every frame
                let mut frame_already_overdue:bool = false;
                let mut video_frame_counter_last:Option<u32> = None;
                let mut missed_deadline_count:u32 = 0;
                let mut missed_deadline_already = false;
            }
        }

        // Remember: Vsync occurs slightly after vblank, so vblank tells us when it's safe to draw
        // and vsync tells us when we missed a frame. For this test app, we'll begin by clearing
        // vblank_triggered, forcing one complete frame to pass us by.
        // This would be suboptimal in a real app but is okay here:
        let _ = peripherals.APF_VIDEO.video.read();

        loop {
            // Busy loop until VBLANK begins, signaling next frame ready to go.
        	loop {
                let video = peripherals.APF_VIDEO.video.read();
                let frame_ready = video.vblank_triggered().bit();

                // Complex tracking to see if frames were skipped
                cfg_if::cfg_if! {
                    if #[cfg(feature = "speed-debug")] {
                        let frame_ready = frame_ready || frame_already_overdue;
                        if frame_ready {
                            let video_frame_counter = video.frame_counter().bits();
                            if let Some(video_frame_counter_last) = video_frame_counter_last {
                                let gap = video_frame_counter as i32 - video_frame_counter_last as i32;
                                if gap > 1 {
                                    if 0== missed_deadline_count % SPEED_DEBUG_RATE {
                                        println!("Too slow! Dropped an entire frame (frames missing {}; fail #{})", gap-1, missed_deadline_count);
                                    }
                                    missed_deadline_count += 1;
                                } else {
                                    if missed_deadline_already { missed_deadline_count += 1 } // We noticed a miss but haven't recorded it
                                    if gap <= 0 {
                                        println!("Catastrophic failure: Video counts no frames between frames (gap of {})", gap);
                                    }
                                }
                            }
                            video_frame_counter_last = Some(video_frame_counter);
                            frame_already_overdue = false;
                            missed_deadline_already = false;
                        }
                    }
                }

                if frame_ready { break; }
            }

            // Controls
            let cont1_key = peripherals.APF_INPUT.cont1_key.read().bits() as u16; // Crop out analog sticks
            let cont1_key_edge = (!cont1_key_last) & cont1_key;
            cont1_key_last = cont1_key;

            use PocketControls::*;

            // Pause
            if cont1_key_edge & FaceSelect as u16 != 0 {
                paused = !paused;
            }

            let flicker_width:usize = DISPLAY_WIDTH / (if cont1_key & DpadLeft as u16 != 0 { 16 }
                else if cont1_key & DpadUp as u16 != 0 { 8 }
                else if cont1_key & DpadRight as u16 != 0 { 4 }
                else if cont1_key & DpadDown as u16 != 0 { 2 }
                else { 1 });

            let flicker_freq:u8 = if cont1_key & FaceB as u16 != 0 { 0b1 }
                else if cont1_key & FaceA as u16 != 0 { 0b10 }
                else if cont1_key & FaceX as u16 != 0 { 0b1000 }
                else { 0 };
            let flickering = (cont1_key & FaceY as u16 != 0) || 0 != frame_counter & flicker_freq;

            unsafe { peripherals.APF_AUDIO.playback_en.write(|w| w.bits(1)) };

            // Video gen
            fn gray_to_565(gray:u8) -> u16 {
                let gray:u16 = gray as u16;
                let mut color:u16 = 0;
                color |= (gray>>3);
                color <<= 5;
                color |= (gray>>2);
                color <<= 6;
                color |= (gray>>3);
                color
            }

            // There's some kind of bug here where the color will flicker black once per cycle,
            // but the point of the app is to flicker, so I'm not fixing it.
            let color_gray:u8 = frame_counter;
            let color_gray = if color_gray > 128 { (0 as u8).wrapping_sub(color_gray) } else { color_gray };
            let color_gray:u8 = if flickering { !color_gray } else { color_gray };
            let color_gray:u16 = gray_to_565(color_gray);

            for y in 0..DISPLAY_HEIGHT {
                for x in 0..flicker_width {
                     *pixel(fb, x, y) = color_gray;
                }
            }

            #[cfg(feature = "speed-debug")]
            {
                let video = peripherals.APF_VIDEO.video.read();
                if !video.vblank_status().bit() { // Status has already gone low
                    if 0== missed_deadline_count % SPEED_DEBUG_RATE {
                        println!("Too slow! Drawing finished outside vblank deadline (fail #{})", missed_deadline_count);
                    }
                    missed_deadline_already = true;
                }
                frame_already_overdue = video.vblank_triggered().bit();
            }

            let audio_needed = AUDIO_TARGET - peripherals.APF_AUDIO.buffer_fill.read().bits() as i32;
            for _ in 0..audio_needed { // 800 samples = 1/60 of a second. This will pause us long enough for a frame to pass
                let value:u32 = wave as u32;
                let value = value >> 4;
                let value = value | (value << 16);

                if (!paused) {
                    let freq_delta = if flickering { FREQ_DELTA * 4 } else { FREQ_DELTA };
                    wave = wave.wrapping_add(freq_delta);
                }

                unsafe { peripherals.APF_AUDIO.out.write(|w| w.bits(value)) };
            }

            // Progress
            if (!paused) {
                frame_counter = frame_counter.wrapping_add(1);
            }
        }
    }

    // Unreachable
}
