#![no_std]
#![no_main]
#![allow(unused_parens)]

use alloc::format;
use core::arch::asm;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::panic::PanicInfo;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::sync::atomic::AtomicU32;
use core::time::Duration;
use core::cell::UnsafeCell;
use embedded_hal::prelude::_embedded_hal_blocking_serial_Write;
use num_traits::float::FloatCore;

extern crate alloc;

use alloc::{boxed::Box, rc::Rc};
use embedded_alloc::Heap;
use litex_hal as hal;
use litex_pac as pac;
use litex_openfpga::*;
use riscv_rt::entry;

// Definition is required for uart_printer.rs to work
hal::uart! {
    UART: pac::UART,
}

#[repr(u16)]
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

const CLOCK_SPEED: u32 = 51_600_000; // FIXME INCORRECT
const CYCLE_PERIOD_NANOS: f64 = 1_000_000_000.0 / (CLOCK_SPEED as f64);

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

fn combine_u32(low: u32, high: u32) -> u64 {
    ((high as u64) << 32) | (low as u64)
}

fn get_cycle_count() -> u64 {
    let peripherals = unsafe { pac::Peripherals::steal() };

    unsafe {
        // Grab cycle count
        peripherals.TIMER0.uptime_latch.write(|w| w.bits(1));
    };

    let low_bits = peripherals.TIMER0.uptime_cycles0.read().bits();
    // let low_bits = unsafe { TEST_ADDR.read_volatile() };
    // println!("{low_bits}");
    let high_bits = peripherals.TIMER0.uptime_cycles1.read().bits();
    let uptime_cycles = combine_u32(low_bits, high_bits);

    // let prev_uptime_cycles_low =
    //     LAST_UPTIME_CYCLES_LOW.load(core::sync::atomic::Ordering::Acquire);
    // let prev_uptime_cycles_high =
    //     LAST_UPTIME_CYCLES_HIGH.load(core::sync::atomic::Ordering::Acquire);

    // let prev_uptime_cycles = combine_u32(prev_uptime_cycles_low, prev_uptime_cycles_high);

    // // Should always fit in u32
    // let cycle_duration = (uptime_cycles - prev_uptime_cycles) as u32;

    (CYCLE_PERIOD_NANOS * (uptime_cycles as f64)).floor() as u64
}

fn render_init(framebuffer_address: *mut u16) {
    let buffer = unsafe { from_raw_parts_mut(framebuffer_address, DISPLAY_WIDTH * DISPLAY_HEIGHT) };

    const PIXEL_MAX:usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
    let mut pixel_offset:usize = 0;
    let pixel_buffer = unsafe { from_raw_parts_mut(framebuffer_address, DISPLAY_WIDTH * DISPLAY_HEIGHT) };
    for idx in 0..PIXEL_MAX {
        pixel_buffer[idx] = 0xFFFF;
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

    println!("-- Minibreak --");

    let fb:*mut u16 = peripherals.VIDEO_FRAMEBUFFER.dma_base.read().bits() as *mut u16;

    render_init(fb);

    // for x in 0..DISPLAY_WIDTH {
    //     for y in 0..DISPLAY_HEIGHT {
    //         *pixel(fb, x, y) = 0xFFFF;
    //     }
    // }

    // for x in 0..DISPLAY_WIDTH {
    //     match x % 10 {
    //         0 => {
    //             // Blue column of 10
    //             let y_max = if x % 100 == 0 {
    //                 // 100 boundary
    //                 20
    //             } else {
    //                 10
    //             };

    //             for y in 0..y_max {
    //                 *pixel(fb, x, y) = 0x001F;
    //             }
    //         }
    //         2 | 4 | 6 | 8 => {
    //             // Even tick of black
    //             for y in 0..5 {
    //                 *fb, pixel(x, y) = 0x0000;
    //             }
    //         }
    //         _ => {
    //             // Bottom line
    //             *fb, pixel(x, 0) = 0x0000;
    //         }
    //     }
    // }

    // loop {}

    // "APP"
    {
        use glam::IVec2;
        use alloc::vec::Vec;

        // Config
        let config_chaos = 0; // 0-2 inclusive
        const CONFIG_IMMORTAL:bool = false;
        const LFO_MAX:u16 = 48000;

        // Basic
        let mut wave:u16 = 0;
        let mut lfo:u16 = LFO_MAX/4;
        const FREQ_DELTA:u16 = 150;
        let mut frame_counter:u8 = 0;
        let mut first_frame = true;

        let mut paused = false;
        let mut dead = false;
        let mut won = false;
        let mut cont1_key_last = 0;
        let mut bleep_high = false;
        let mut bleeping = 0;
        let mut blooping = 0;
        let mut final_vader_facing = 1;

        // Geometry support: 0,0 is top left
        fn ivec2_within(size:IVec2, at:IVec2) -> bool {
            IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
        }
        fn ivec2_le(left:IVec2, right:IVec2) -> bool {
            left.cmple(right).all()
        }
        fn ivec2_lt(left:IVec2, right:IVec2) -> bool { // Unused
            left.cmplt(right).all()
        }
        fn ivec2_ge(left:IVec2, right:IVec2) -> bool {
            left.cmpge(right).all()
        }
        fn ivec2_gt(left:IVec2, right:IVec2) -> bool {
            left.cmpgt(right).all()
        }
        #[derive(Debug, Clone, Copy)]
        struct IRect2 { // br is non-inclusive
            ul: IVec2,  // Upper left
            br: IVec2   // Bottom right
        }
        impl IRect2 {
            fn new(ul:IVec2, br:IVec2) -> Self { Self {ul, br} }
            fn new_centered(center:IVec2, size:IVec2) -> Self {
                let br = center + size/2; // Bias placement toward upper-left
                let ul = br - size;
                Self {ul, br}
            }
            fn within(&self, test:IVec2) -> bool {
                ivec2_le(self.ul, test) && ivec2_gt(self.br, test)
            }
            fn intersect(&self, test:IRect2) -> bool { // Will misbehave on 0-size rects
                self.within(test.ul) || {
                    let in_br = test.br+IVec2::NEG_ONE; // For testing within the point just inside must be in
                    self.within(in_br) || // All 4 corners
                    self.within(IVec2::new(test.ul.x, in_br.y)) ||
                    self.within(IVec2::new(in_br.x, test.ul.y))
                }
            }
            fn enclose(&self, test:IRect2) -> bool {
                ivec2_le(self.ul, test.ul) && ivec2_ge(self.br, test.br) // For testing enclose the rects only need to coincide
            }
            fn size(&self) -> IVec2 {
                self.br - self.ul
            }
            fn center(&self) -> IVec2 {
                (self.br + self.ul)/2
            }
            fn offset(&self, by:IVec2) -> IRect2 {
                return IRect2::new(self.ul + by, self.br + by);
            }
            fn force_enclose_x(&self, test:IRect2) -> IRect2 { // ASSUMES SELF SMALLER THAN TEST
                let mut excess = test.ul.x - self.ul.x;
                if excess > 0 { return self.offset(IVec2::new(excess, 0)) }
                let excess = test.br.x - self.br.x;
                if excess < 0 { return self.offset(IVec2::new(excess, 0)) }
                self.clone()
            }
        }

        if false { // "FUNCTIONAL TESTS"
            let rect = IRect2::new(IVec2::new(5, 5), IVec2::new(15,15));
            for y in 0..3 {
                for x in 0..3 {
                    let v = IVec2::new(x*10,y*10);
                    assert_eq!(rect.within(v), (x==1 && y==1), "Incorrect within! rect: {:?} v: {:?}", rect, v);
                    let r2 = IRect2::new_centered(v, IVec2::ONE*2);
                    assert_eq!(rect.enclose(r2), (x==1 && y==1), "Incorrect enclose! rect: {:?} v: {:?}", rect, r2);
                }
            }
            for y in 0..5 {
                for x in 0..5 {
                    let v = IVec2::new(x*5,y*5);
                    let r2 = IRect2::new_centered(v, IVec2::ONE*2);
                    assert_eq!(rect.intersect(r2), !(x==0 || x==4 || y==0 || y==4), "Incorrect intersect! rect: {:?} v: {:?}", rect, r2);
                }
            }
        }

        let screen = IRect2::new(IVec2::ZERO, IVec2::new(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32));

        struct Vader {
            rect:IRect2
        }

        struct Ball {
            rect:IRect2,
            facing:IVec2
        }

        struct Player {
            rect:IRect2,
            facing:i32 // l/r
        }

        let mut vaders: Vec<Vader> = Default::default();
        let mut balls: Vec<Ball> = Default::default();
        let mut players: Vec<Player> = Default::default();

        const PLAYER_SIZE:IVec2 = IVec2::new(40, 8);
        const PLAYER_START:IVec2 = IVec2::new(DISPLAY_WIDTH as i32/2, DISPLAY_HEIGHT as i32-20-PLAYER_SIZE.y/2);
        const PLAYER_COLOR:u16 = 0b11111_101010_11111;
        const PLAYER_SPEED:i32 = 2;

        const BALL_SIZE:IVec2 = IVec2::new(4,4);
        let ball1_start:IVec2 = PLAYER_START + IVec2::new(0, -30);
        const BALL_COLOR:u16 = 0b00000_000000_11111 ^ 0xFFFF;
        const BALL_SPEED:i32 = 3;
        const BALL_FACING:IVec2 = IVec2::new(1,-1);

        const REFLECTS:[IVec2;2] = [ IVec2::new(-1,1), IVec2::new(1,-1) ];
        const REFLECT_BLEEP:u16 = 800*2;

        const DEATH_BLOOP_STROBE:u16 = 8*800;
        const DEATH_BLOOP:u16 = DEATH_BLOOP_STROBE*6;

        const VADER_COLS:i32 = 8;
        const VADER_ROWS:i32 = 4;
        const VADER_SIZE:IVec2 = IVec2::new(20, 12);
        const VADER_PADDING:IVec2 = IVec2::new(10,20);
        const VADER_ORIGIN:IVec2 = IVec2::new((DISPLAY_WIDTH as i32-(VADER_COLS*VADER_SIZE.x + (VADER_COLS-1)*VADER_PADDING.x))/2, 20);
        const VADER_COLOR:u16 = 0b11111_000000_00000 ^ 0xFFFF;

        assert_eq!(VADER_ORIGIN.x+VADER_PADDING.x >= 0, true, "Screen too narrow for vaders");

        players.push(Player { rect:IRect2::new_centered(PLAYER_START, PLAYER_SIZE), facing:0 });

        let ball_facing = { // Randomly start moving left or right; use the current UTC as a very weak RNG
            let mut ball_facing = BALL_FACING;
            if 0 == peripherals.APF_RTC.unix_seconds.read().bits() % 2 { ball_facing.x *= -1 }
            ball_facing
        };
        balls.push(Ball { rect:IRect2::new_centered(ball1_start, BALL_SIZE), facing:ball_facing });

        for y in 0..VADER_ROWS {
            for x in 0..VADER_COLS {
                let ul = VADER_ORIGIN + IVec2::new(x, y)*(VADER_SIZE + VADER_PADDING);
                vaders.push(Vader { rect:IRect2::new(ul, ul+VADER_SIZE) });
            }
        }

        // Gfx support

        fn fill(fb: *mut u16, rect:IRect2, color:u16) {
            for y in rect.ul.y..rect.br.y {
                for x in rect.ul.x..rect.br.x {
                    *pixel(fb, x as usize,y as usize) ^= color; // Assume positive
                }
            }
        }

        // Initial draw
        for player in &players { fill(fb, player.rect, PLAYER_COLOR); }
        for ball in &balls { fill(fb, ball.rect, BALL_COLOR); }
        for vader in &vaders { fill(fb, vader.rect, VADER_COLOR); }

        loop {
            let mut buffer_fill = peripherals.APF_AUDIO.buffer_fill.read().bits();

            // Wait for frame
            while buffer_fill > 400 {
                // Busy wait until the buffer is half empty
                buffer_fill = peripherals.APF_AUDIO.buffer_fill.read().bits();
            }

            // Controls
            let cont1_key = peripherals.APF_INPUT.cont1_key.read().bits() as u16; // Crop out analog sticks
            let cont1_key_edge = (!cont1_key_last) & cont1_key;
            cont1_key_last = cont1_key;

            use PocketControls::*;

            // Pause
            if !dead && cont1_key_edge & FaceSelect as u16 != 0 {
                paused = !paused;
            }

            // Mechanics

            if !paused && !dead && !won {
                // Vader
                // (When one block is left, have it start moving so you aren't stuck unable to hit it.)
                if vaders.len() == 1 {
                    let vader = &mut vaders[0];
                    fill(fb, vader.rect, VADER_COLOR);
                    let vader_move = IVec2::new(final_vader_facing, 0);
                    let rect = vader.rect.offset(vader_move);
                    vader.rect = if screen.enclose(rect) { rect } else {
                        final_vader_facing = -final_vader_facing;
                        vader.rect.offset(-vader_move)
                    };
                    fill(fb, vader.rect, VADER_COLOR);
                }

                // Player
                for player in &mut players {
                    // Controls: Movement
                    const LR_MASK:u16 = DpadLeft as u16 | DpadRight as u16;
                    player.facing = if cont1_key & LR_MASK == LR_MASK { // Impossible on Analogue builtin but who knows about bluetooth
                        if cont1_key_edge & DpadLeft as u16 != 0 { -1 }
                        else if cont1_key_edge & DpadRight as u16 != 0 { 1 }
                        else { player.facing }
                    } else {
                        if cont1_key & DpadLeft as u16 != 0 { -1 }
                        else if cont1_key & DpadRight as u16 != 0 { 1 }
                        else { 0 }
                    };

                    if player.facing != 0 {
                        if config_chaos < 2 {
                            fill(fb, player.rect, PLAYER_COLOR);
                        }

                        player.rect = player.rect.offset(IVec2::new(player.facing*PLAYER_SPEED, 0))
                            .force_enclose_x(screen);

                        fill(fb, player.rect, PLAYER_COLOR);
                    }
                }

                // Ball
                for ball in &mut balls {
                    if config_chaos < 2 {
                        fill(fb, ball.rect, BALL_COLOR);
                    }

                    bleep_high = false;

                    // Step one pixel at a time, one axis at a time.
                    'step: for step in 0..BALL_SPEED {
                        for (aid, axis) in IVec2::AXES.into_iter().enumerate() {
                            let v = axis*ball.facing;
                            //println!("TEST!! {}: {:?} = {:?} * {:?}", step, (aid,axis), ball.facing, v);
                            let rect = ball.rect.offset(v);
                            let mut reflect = false;
                            if !screen.enclose(rect) {
                                if v.y<=0 || CONFIG_IMMORTAL {
                                    reflect = true;
                                } else {
                                    // Touched bottom of screen. Game over.
                                    dead = true;
                                    blooping = DEATH_BLOOP;
                                    break 'step;
                                }
                            }
                            for player in &players {
                                if reflect { break; }
                                if player.rect.intersect(rect) {
                                    reflect = true;
                                    if v.y>0 {
                                        ball.facing.x = if ball.rect.center().x > player.rect.center().x
                                            { 1 } else { -1 }
                                    }
                                }
                            }
                            let mut destroy:Option<usize> = None;
                            for (idx,vader) in vaders.iter().enumerate() {
                                if reflect { break; }
                                if vader.rect.intersect(rect) {
                                    reflect = true;
                                    destroy = Some(idx);
                                    bleep_high = true;
                                    fill(fb, vader.rect, VADER_COLOR);
                                }
                            }
                            ball.rect = if !reflect {
                                rect
                            } else {
                                ball.facing *= REFLECTS[aid];
                                bleeping = REFLECT_BLEEP;
                                ball.rect.offset(-v)
                            };
                            if let Some(idx) = destroy { // Destroy one vader
                                vaders.remove(idx);
                                if vaders.len() == 0 {
                                    won = true;
                                    blooping = DEATH_BLOOP;
                                    break 'step;
                                }
                            }
                        }
                    }

                    fill(fb, ball.rect, BALL_COLOR);
                }
            }

            // Audio gen
            for i in 0..800 { // 800 samples = 1/60 of a second. This will pause us long enough for a frame to pass
                let mut lfo_engaged = false;
                if (!paused) {
                    let freq_delta = if bleeping>0 {
                        bleeping -= 1;
                        if bleep_high { FREQ_DELTA*4 } else { FREQ_DELTA*2 }
                    } else if blooping>0 {
                        blooping -= 1;
                        if blooping == 0 { paused = true; }
                        if 0!=(blooping/DEATH_BLOOP_STROBE)%2 {
                            if !won { FREQ_DELTA/2 } else { FREQ_DELTA*4 }
                        } else { 0 }
                    } else {
                        lfo_engaged = true;
                        FREQ_DELTA
                    };
                    wave = wave.wrapping_add(freq_delta);

                    lfo = (lfo+1)%LFO_MAX;
                }

                let mut value:u32 = wave as u32;
                value = value >> 4;
                if lfo_engaged { value *= lfo as u32; value /= LFO_MAX as u32; }
                value = value | (value << 16);

                unsafe { peripherals.APF_AUDIO.out.write(|w| w.bits(value)) };
            }

            // Progress
            if (!paused) {
                first_frame = false;
            }

            unsafe { peripherals.APF_AUDIO.playback_en.write(|w| w.bits(1)) };
        }
    }

    println!("Finished reading");

    loop {}
}
