#![no_std]
#![cfg_attr(not(test), no_main)]
#![allow(unused_parens)]

use core::panic::PanicInfo;
//use core::slice::from_raw_parts_mut;

extern crate alloc;

use embedded_alloc::Heap;
use litex_hal as hal;
use litex_pac as pac;
use litex_openfpga::*;
use riscv_rt::entry;

mod irect2;

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

const HEAP_SIZE: usize = 1024 * 1024;
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
#[cfg_attr(not(test), entry)] // FIXME: Retest this after Cargo issue #13146 addressed
fn main() -> ! {
    use alloc::boxed::Box;

    let peripherals = unsafe { pac::Peripherals::steal() };

    // Initialize the allocator BEFORE you use it
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) };

    println!("-- Sprite test --");

    // Framebuffer pointer
    // Note we also had the option of simply picking an address and writing dma_base instead of reading it
    const DISPLAY_LEN:usize = DISPLAY_HEIGHT*DISPLAY_WIDTH;
    let mut screens = [Box::new([0 as u16; DISPLAY_LEN]), Box::new([0 as u16; DISPLAY_LEN])];
    let mut fullscreen_dirty = [true, true];
    let mut screen_current = 0; // First frame or two will be pretty nonsense

//    render_init(fb);

    // "APP"
    {
        use alloc::vec::Vec;
        use glam::IVec2;
        use crate::irect2::*;

        // Top-level config

        // Basic state

        const SELECT_BLINK_STANDARD:i32 = 40;
        const SELECT_BLINK_MODULUS:i32 = 5;

        let mut paused = false;
        let dead = false;
        let mut cont1_key_last = 0; // State of controller on previous loop
        // let mut first_frame = true;

        // UI
        let mut select_blink_remain = 0;
        let mut select_idx;

        // Display

        cfg_if::cfg_if! {
            if #[cfg(feature = "speed-debug")] { // State used to detect deadline misses
                const SPEED_DEBUG_RATE:u32 = 1; // Every frame
                let mut frame_already_overdue:bool = false;
                let mut video_frame_counter_last:Option<u32> = None;
                let mut missed_deadline_count:u32 = 0;
                let mut missed_deadline_already = false;
                let mut frame_deadline_state;
                let mut frame_deadline_state_was = [0,0]; // Initial values will be unread
                let mut frame_deadline_state_changed = false; // Initial value will be unread
            }
        }

        let display = IVec2::new(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32);
        let display_rect = IRect2::new(IVec2::ZERO, display);

        // Audio properties

        const AUDIO_TARGET:i32 = 48000/60 + 200; // Try to always fill audio buffer to this point

        // Audio comes in the form of a single sawwave generator that can be reset or int-pitchshifted

        const AUDIO_FREQ_DELTA:u16 = 75; // Step of basic sawtooth wave; increase to increase pitch of bleeps/bloops
        const AUDIO_BLEEP_LEN:u16 = 800*2; // How long (in samples) does a single bleep last?

        // Audio state

        let mut audio_wave:u16 = 0; // Sawtooth wave state used for all sounds
        let mut audio_bleeping = 0; // Remaining samples to play bleep
        let mut audio_pitch_mod = 4; // Pitch multiplier. x2 for one octave

        // Game properties

        // Game state

        struct RawImage {
            w:u16, h:u16,
            pixels: *const u16,
            flippable: bool
        }

        struct Sprite {
            idx:usize, at:IVec2, reversed:bool, stopped:bool, dead:bool, dirty:[Option<IVec2>;2]
        }
        impl Sprite {
            pub fn new(idx:usize, at:IVec2, reversed:bool) -> Sprite {
                Sprite { idx, at, reversed, stopped:false, dead:false, dirty:[None, None] }
            }
        }

        let playfield = RawImage { w:256, h:192, flippable:false, pixels: include_bytes!(concat!(env!("OUT_DIR"), "/playfield_bg.bin")) as *const u8 as _ };
        let playfield_size = IVec2::new(playfield.w as i32, playfield.h as i32);
        let playfield_basis = (display - playfield_size) / 2;

        let witch = RawImage { w:30, h:30, flippable: true, pixels: include_bytes!(concat!(env!("OUT_DIR"), "/player_hit.bin")) as *const u8 as _ };
        let blobber = RawImage { w:34, h:34, flippable: false, pixels: include_bytes!(concat!(env!("OUT_DIR"), "/blobber_attack.bin")) as *const u8 as _ };

        let sprite_data = [witch, blobber];

        #[cfg(feature = "speed-debug")]
        let speed_debug_rect = {
            let margin = playfield_basis.y/3;
            let ul = IVec2::new(0, margin*4 + playfield_size.y);
            IRect2::new(ul, ul + IVec2::new(DISPLAY_WIDTH as i32, margin))
        };

        let mut sprites: Vec<Sprite> = Default::default();

        sprites.push(Sprite::new(1, IVec2::ZERO, false)); // idx, location, reversed, flippable
        sprites.push(Sprite::new(0, display/2, false));

        select_idx = sprites.len() - 1;

        let mut dirty_temp:Vec<IRect2> = Default::default();

        loop {
            // Busy loop until VBLANK begins, signaling next frame ready to go.
            // We'd like to do all drawing inside VBLANK to prevent tearing.
            loop {
                let video = peripherals.APF_VIDEO.video.read();
                let frame_ready = video.vblank_triggered().bit();

                // Complex tracking to see if frames were skipped
                cfg_if::cfg_if! {
                    if #[cfg(feature = "speed-debug")] {
                        let frame_ready = frame_ready || frame_already_overdue;
                        frame_deadline_state = 0;
                        if frame_ready {
                            let video_frame_counter = video.frame_counter().bits();
                            if let Some(video_frame_counter_last) = video_frame_counter_last {
                                let gap = video_frame_counter as i32 - video_frame_counter_last as i32;
                                if gap > 1 {
                                    if 0== missed_deadline_count % SPEED_DEBUG_RATE {
                                        frame_deadline_state = 2;
                                        #[cfg(feature = "speed-debug-serial")]
                                        println!("Too slow! Dropped an entire frame (frames missing {}; fail #{})", gap-1, missed_deadline_count);
                                    }
                                    missed_deadline_count += 1;
                                } else {
                                    if missed_deadline_already { missed_deadline_count += 1 }
                                    if gap <= 0 {
                                        frame_deadline_state = 1;
                                        #[cfg(feature = "speed-debug-serial")]
                                        println!("Catastrophic failure: Video counts no frames between frames (gap of {})", gap);
                                    }
                                }
                            }
                            video_frame_counter_last = Some(video_frame_counter);
                            frame_already_overdue = false;
                            missed_deadline_already = false;

                            // For speed debug rectangle 
                            frame_deadline_state_changed = frame_deadline_state != frame_deadline_state_was[screen_current];
                            frame_deadline_state_was[screen_current] = frame_deadline_state;
                        }
                    }
                }

                if frame_ready { break; }
            }

            // Swap read/write buffer

            unsafe { // This will be u64, not u32, on a LP64 system
                peripherals.VIDEO_FRAMEBUFFER.dma_base.write(|w| w.bits( screens[screen_current].as_ptr() as u32 ));
            }
            if !paused {
                screen_current = (screen_current + 1)%2;
            }

            #[cfg(feature = "speed-debug")]
            {
                let video = peripherals.APF_VIDEO.video.read();
                if !video.vblank_status().bit() { // Status has already gone low
                    if 0== missed_deadline_count % SPEED_DEBUG_RATE {
                        #[cfg(feature = "speed-debug-serial")]
                        println!("Too slow! Drawing finished outside vblank deadline (fail #{})", missed_deadline_count);
                    }
                    missed_deadline_already = true;
                }
                frame_already_overdue = video.vblank_triggered().bit();
            }

            // Controls

            let cont1_key = peripherals.APF_INPUT.cont1_key.read().bits() as u16; // Bitmask (crop out analog sticks)
            let cont1_key_edge = (!cont1_key_last) & cont1_key; // Bitmask is 1 iff a button press is *new this frame*
            cont1_key_last = cont1_key;

            use PocketControls::*;

            // Controls: Reset
            if cont1_key_edge & FaceStart as u16 != 0 {
                unsafe { peripherals.CTRL.reset.write(|w| w.bits(1)); } // 1 resets entire SOC
            }

            // Controls: Select highlight
            if cont1_key_edge & TrigL1 as u16 != 0 {
                select_blink_remain = SELECT_BLINK_STANDARD;
            }

            // Controls: Directly control selected (toggle)
            let select_control = cont1_key & (FaceY as u16) != 0;

            if select_control {
                // Directly control selected
                let selected = &mut sprites[select_idx];
                if cont1_key & (DpadUp as u16) != 0 { selected.at += IVec2::new(0,-1); }
                if cont1_key & (DpadDown as u16) != 0 { selected.at += IVec2::new(0,1); }
                if cont1_key & (DpadLeft as u16) != 0 { selected.at += IVec2::new(-1,0); }
                if cont1_key & (DpadRight as u16) != 0 { selected.at += IVec2::new(1,0); }
            } else {
                if cont1_key_edge & (DpadLeft as u16) != 0 {
                    if select_idx > 0 { select_idx -= 1; } // No underflow
                    select_blink_remain = SELECT_BLINK_STANDARD;
                }
                if cont1_key_edge & (DpadRight as u16) != 0 {
                    let (old_idx, old_at) = {
                        let selected = &sprites[select_idx];
                        (selected.idx, selected.at)
                    };
                    select_idx += 1;
                    select_blink_remain = SELECT_BLINK_STANDARD;
                    if select_idx >= sprites.len() { // Spawn new on overflow
                        let h32 = DISPLAY_HEIGHT as i32;
                        sprites.push(Sprite::new((old_idx+1)%sprite_data.len(), IVec2::new(old_at.x, (old_at.y + h32*7/4) % h32), false));
                        audio_bleeping = AUDIO_BLEEP_LEN;
                        audio_pitch_mod = 16;
                        select_blink_remain += SELECT_BLINK_MODULUS;
                        println!("Spawned: Count {}", sprites.len());
                    }
                }
                if cont1_key_edge & (DpadUp as u16 | DpadDown as u16) != 0 {
                    let selected = &mut sprites[select_idx];
                    let dir:isize = if cont1_key_edge & DpadUp as u16 != 0 { -1 } else { 1 };
                    selected.idx = (selected.idx as isize + dir).rem_euclid(sprite_data.len() as isize) as usize;

                    audio_bleeping = AUDIO_BLEEP_LEN;
                    audio_pitch_mod = 8;
                }
            }

            // Controls: Halt selected
            if cont1_key_edge & (FaceX as u16) != 0 {
                let selected = &mut sprites[select_idx];
                selected.stopped = !selected.stopped;

                audio_bleeping = AUDIO_BLEEP_LEN;
                audio_pitch_mod = 8;
            }

            // Controls: Reset selected
            if cont1_key_edge & (FaceA as u16) != 0 {
                let selected = &mut sprites[select_idx];
                selected.at = IVec2::ZERO;
                selected.reversed = false;

                audio_bleeping = AUDIO_BLEEP_LEN;
                audio_pitch_mod = 2;
            }

            // Controls: Destroy selected
            if cont1_key_edge & (FaceB as u16) != 0 {
                let mut live_count = 0;
                for Sprite { dead, .. } in sprites.iter() { if !dead { live_count += 1; } }
                audio_bleeping = AUDIO_BLEEP_LEN;
                if live_count > 1 {
                    let selected = &mut sprites[select_idx];
                    selected.dead = true;
                    audio_pitch_mod = 2;
                } else {
                    // Denied
                    audio_pitch_mod = 1;
                }
            }

            // Note: Pause controls happen late (to allow next frame to complete drawing)

            // Mechanics

            // Draw next frame

            let screen = &mut* screens[screen_current];

            // Clear dirty rectangles
            if fullscreen_dirty[screen_current] { // First pass full-screen draw
                dirty_temp.push(display_rect);
                fullscreen_dirty[screen_current] = false;
            } else {
                if (!paused) {
                    for Sprite {idx:sprite_idx, dirty, ..} in sprites.iter() {
                        if let Some(at) = dirty[screen_current] {
                            let size = {
                                let sprite = &sprite_data[*sprite_idx];
                                IVec2::new(sprite.w as i32, sprite.h as i32)
                            };
                            let rect = display_rect.overlap(IRect2::new(at, at+size));
                            if let Some(rect) = rect {
                                dirty_temp.push(rect);
                            }
                            // Assume if we're already drawing, we DON'T need to clear dirty-- it will clear soon
                        }
                    }
                }
                #[cfg(feature = "speed-debug")] // Note if this coincides with pause it will act weird. For debug code that's ok
                if frame_deadline_state_changed {
                    dirty_temp.push(speed_debug_rect);
                }
            }
            for rect in dirty_temp.iter() {
                let background = 0;
                #[cfg(feature = "speed-debug")]
                let speed_debug_background = match frame_deadline_state {
                    1 => 0x6800,
                    2 => 0xF800,
                    _ => background
                };
                for y in rect.ul.y..rect.br.y {
                    for x in rect.ul.x..rect.br.x {
                        let at = IVec2::new(x, y) - playfield_basis;
                        screen[y as usize * DISPLAY_WIDTH + x as usize] =
                            if (ivec2_within(playfield_size, at)) {
                                unsafe {
                                    *playfield.pixels.wrapping_add((at.y * playfield_size.x + at.x) as usize)
                                }
                            } else {
                                #[cfg(feature = "speed-debug")]
                                let background = if speed_debug_background != background && speed_debug_rect.within(IVec2::new(x as i32, y as i32)) { speed_debug_background } else { background };
                                background
                            }
                    }
                }
            }
            dirty_temp.clear(); // Reuse one vector so we're not reallocating

            // Draw sprites
            if (!paused) {
                let select_blink_active =
                    if select_blink_remain > 0 {
                        select_blink_remain -= 1;
                        (select_blink_remain / SELECT_BLINK_MODULUS) % 2 == 0
                    } else { false };

                let mut any_dead = false;

                for (idx, Sprite {idx: sprite_idx, at, reversed, dead, dirty, stopped}) in sprites.iter_mut().enumerate() {
                    let sprite = &sprite_data[*sprite_idx];
                    let blinking = select_blink_active && idx == select_idx || *dead;
                    if !blinking {
                        let transparent = unsafe { *sprite.pixels };
                        for y in 0..sprite.h {
                            for x in 0..sprite.w {
                                let pix_at = *at + IVec2::new(x as i32, y as i32);
                                if (ivec2_within(display, pix_at)) {
                                    // WARNING: u16 MATH COULD OVERFLOW
                                    let color = unsafe { *sprite.pixels.wrapping_add((y * sprite.w + if *reversed && sprite.flippable { sprite.w - x - 1 } else { x } ) as usize) };
                                    if (color != transparent) {
                                        screen[pix_at.y as usize * DISPLAY_WIDTH + pix_at.x as usize] = color;
                                    }
                                }
                            }
                        }
                        dirty[screen_current] = Some(*at);
                    } else {
                        dirty[screen_current] = None;
                    }
                    if !(*stopped || *dead) {
                        let mut flip = false;
                        if *reversed {
                            *at += IVec2::new(-1, if at.x%2==0 { 1 } else { 0 });
                            if at.x <= 0 {
                                flip = true;
                            }
                        } else {
                            *at += IVec2::new(1, 0);
                            if at.x + sprite.w as i32 >= DISPLAY_WIDTH as i32 {
                                flip = true;
                            }
                        }
                        if (flip) {
                            *reversed = !*reversed;
                            audio_bleeping = AUDIO_BLEEP_LEN;
                            if at.y + sprite.h as i32 >= DISPLAY_HEIGHT as i32 {
                                *at = IVec2::new(0, at.y - DISPLAY_HEIGHT as i32);
                                audio_pitch_mod = 2;

                                // Uncomment this next line to test the speed test rectangle. Yes, this is silly
                                // for _ in 1..1000000 { audio_pitch_mod *= 3 }
                            } else {
                                audio_pitch_mod = 4;
                            }
                        }
                    }
                    if *dead { any_dead = true }
                }

                if any_dead {
                    let mut max = sprites.len();
                    let mut idx = 0;
                    while idx < max {
                        if sprites[idx].dead {
                            if select_idx > idx { select_idx -= 1 }
                            max -= 1;
                            sprites.remove(idx);
                        } else {
                            idx += 1;
                        }
                    }
                    select_idx = core::cmp::min(select_idx, sprites.len()-1);
                }
            }

            // Audio generation

            // Generate enough samples to fill us up to our desired buffer (a frame plus a safety margin)
            let audio_needed = AUDIO_TARGET - peripherals.APF_AUDIO.buffer_fill.read().bits() as i32;
            for _ in 0..audio_needed {
                if audio_bleeping>0 { // Currently bleeping. Note: Does NOT depend on paused
                    audio_bleeping -= 1;

                    // freq_delta will determine the frequency of the saw generator this sample
                    let freq_delta = AUDIO_FREQ_DELTA * audio_pitch_mod;

                    // Simplest waveform possible: Increment last sample's value by the delta, then wrap around at 2^16
                    audio_wave = audio_wave.wrapping_add(freq_delta);
                }

                let mut value:u32 = audio_wave as u32;

                // Max volume is 2^12-1
                value = value >> 4;

                // Output value is two stereo i16s packed into one u32
                // Notice we did our math above in u32; it doesn't matter because bit 15 is always 0
                value = value | (value << 16);

                unsafe { peripherals.APF_AUDIO.out.write(|w| w.bits(value)) };
            }

            unsafe { peripherals.APF_AUDIO.playback_en.write(|w| w.bits(1)) };

            // Late mechanics

            // Controls: Pause // TODO move check earlier
            if !dead && cont1_key_edge & FaceSelect as u16 != 0 {
                paused = !paused;
            }

            // Uncomment if you need to know if you're on the first frame
            // if (!paused) {
            //     first_frame = false;
            // }
        }
    }

    // Unreachable
}
