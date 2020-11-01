#![feature(slice_fill)]
#![no_std]
#![no_main]

extern crate panic_halt;

use hifive1::hal::i2c::{I2c, Speed};
use hifive1::hal::prelude::*;
use hifive1::hal::DeviceResources;
use hifive1::{pin, sprintln};
use riscv_rt::entry;

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
    let mut data = [0x00; 8192 + 1];
    data[0] = 0b01000000; // the control byte
    if let Err(err) = i2c.write(address, &data) {
        sprintln!("Error: {:?}", err);
    }

    // dimensions of the frames
    let width = 128;
    let height = 128;

    // prepare drawing area
    write_cmd!(0x15, 0, width / 2 - 1);
    write_cmd!(0x75, 0, height - 1);

    // a not-so-small macro to help us draw an image
    let mut fill_screen = |color: u8| {
        let color = (color << 4) + color;
        data.fill(color);
        data[0] = 0b01000000;

        if let Err(err) = i2c.write(address, &data[..]) {
            sprintln!("Error: {:?}", err);
        }
        let _ = blue.toggle();
    };

    loop {
        fill_screen(0x00);
        fill_screen(0x0f);
    }
}
