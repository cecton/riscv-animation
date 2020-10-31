#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::i2c::{I2c, Speed};
use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv_rt::entry;

const FRAME_1: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-1.raw"));
const FRAME_2: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-2.raw"));
const FRAME_3: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-3.raw"));
const FRAME_4: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-4.raw"));
const FRAME_5: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-5.raw"));
const FRAME_6: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-6.raw"));
const FRAME_7: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-7.raw"));
const FRAME_8: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-8.raw"));
const FRAME_9: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-9.raw"));
const FRAME_10: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-10.raw"));
const FRAME_11: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-11.raw"));
const FRAME_12: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-12.raw"));
const FRAME_13: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-13.raw"));
const FRAME_14: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-14.raw"));
const FRAME_15: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/F501-15.raw"));

#[entry]
fn main() -> ! {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;
    let pins = dr.pins;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());

    // Configure UART for stdout
    hifive1::stdout::configure(
        p.UART0,
        pin!(pins, uart0_tx),
        pin!(pins, uart0_rx),
        115_200.bps(),
        clocks,
    );

    // Configure I2C
    let sda = pin!(pins, i2c0_sda).into_iof0();
    let scl = pin!(pins, i2c0_scl).into_iof0();
    let mut i2c = I2c::new(p.I2C0, sda, scl, Speed::Fast, clocks);

    // Get blue led
    let sck = pin!(pins, spi0_sck);
    let mut blue = sck.into_inverted_output();

    let address = 0b0111100; // replace this by the address of your device

    // a small macro to help us send commands without repeating ourselves too much
    macro_rules! write_cmd {
        ($($bytes:expr),+) => {{
            if let Err(err) = i2c.write(address, &[0b00000000, $($bytes),+]) {
                sprintln!("Error: {:?}", err);
            }
        }};
    }

    // turn on the screen
    write_cmd!(0xae);
    write_cmd!(0xaf);
    write_cmd!(0xa0, 0x51);

    // fill the screen
    // our screen is 128 pixels long but we divide by 2 because there are 2 pixels per byte
    write_cmd!(0x15, 0, 63);
    // our screen is 128 pixels height
    write_cmd!(0x75, 0, 127);
    // note that I reduced the buffer size!!
    let mut data = [0x00; 1024 + 1];
    data[0] = 0b01000000; // the control byte
    for _ in 0..8 {
        if let Err(err) = i2c.write(address, &data) {
            sprintln!("Error: {:?}", err);
        }
    }

    // dimensions of the frames
    let width = 40;
    let height = 42;

    // prepare drawing area
    write_cmd!(0x15, 0, width / 2 - 1);
    write_cmd!(0x75, 0, height - 1);

    // we override the first data byte with the control byte which tells the screen we are
    // sending data
    //
    // note: it was done already before but just want to make sure in case you comment the screen
    // filling above
    data[0] = 0b01000000;

    // a not-so-small macro to help us draw an image
    macro_rules! draw_frame {
        ($frame:expr) => {{
            // an iterator that will convert the frame's bytes to data bytes usable by the screen:
            //
            // every byte sent to the screen draws 2 pixels: the first 4 bits are for the first
            // pixel while the last 4 bits are for the second pixel
            //
            // every byte in the frame contains 8 bits so 8 monochromatic pixels
            //
            // 8 / 2 = 4
            //
            // this iterator returns 4 data bytes for 1 frame byte
            let mut chunks = $frame.iter().map(|x| {
                [
                    (x & 0b10000000).count_ones() as u8 * 0b11110000
                        + (x & 0b01000000).count_ones() as u8 * 0b00001111,
                    (x & 0b00100000).count_ones() as u8 * 0b11110000
                        + (x & 0b00010000).count_ones() as u8 * 0b00001111,
                    (x & 0b00001000).count_ones() as u8 * 0b11110000
                        + (x & 0b00000100).count_ones() as u8 * 0b00001111,
                    (x & 0b00000010).count_ones() as u8 * 0b11110000
                        + (x & 0b00000001).count_ones() as u8 * 0b00001111,
                ]
            });
            // we count the number of bytes that have been copied so we don't send the whole buffer
            let mut i = 1;
            while let Some(chunk) = chunks.next() {
                // copy_from_slice requires that the source slice and the destination slice are
                // exactly the same otherwise it will panic
                data[i..(i + 4)].copy_from_slice(&chunk);
                i += 4;
            }

            if let Err(err) = i2c.write(address, &data[..i]) {
                sprintln!("Error: {:?}", err);
            }
            let _ = blue.toggle();
        }};
    }

    loop {
        draw_frame!(FRAME_1);
        draw_frame!(FRAME_2);
        draw_frame!(FRAME_3);
        draw_frame!(FRAME_4);
        draw_frame!(FRAME_5);
        draw_frame!(FRAME_6);
        draw_frame!(FRAME_7);
        draw_frame!(FRAME_8);
        draw_frame!(FRAME_9);
        draw_frame!(FRAME_10);
        draw_frame!(FRAME_11);
        draw_frame!(FRAME_12);
        draw_frame!(FRAME_13);
        draw_frame!(FRAME_14);
        draw_frame!(FRAME_15);
    }
}
