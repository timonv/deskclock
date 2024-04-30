use std::error::Error;

use epd_waveshare::epd2in7b::{Display2in7b, Epd2in7b};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal::delay::DelayNs;
use epd_waveshare::{color::*, graphics::DisplayRotation, prelude::*};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    SPIError, SpidevDevice, SysfsPin,
};

use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, SimpleHalSpiDevice, SlaveSelect, Spi},
};
use rppal::{spi::Mode, system::DeviceInfo};

const BUSY: u8 = 24;
const DC: u8 = 25;
const RST: u8 = 17;
const CS: u8 = 26;

fn main() -> Result<(), Box<dyn Error>> {
    // let busy = SysfsPin::new(24); // GPIO 24, board J-18
    // busy.export().expect("busy export");
    // while !busy.is_exported() {}
    // busy.set_direction(Direction::In).expect("busy Direction");
    //
    // let dc = SysfsPin::new(25); // GPIO 25, board J-22
    // dc.export().expect("dc export");
    // while !dc.is_exported() {}
    // dc.set_direction(Direction::Out).expect("dc Direction");
    // // dc.set_value(1).expect("dc Value set to 1");
    //
    // let rst = SysfsPin::new(17); // GPIO 17, board J-11
    // rst.export().expect("rst export");
    // while !rst.is_exported() {}
    // rst.set_direction(Direction::Out).expect("rst Direction");
    // // rst.set_value(1).expect("rst Value set to 1");
    //
    // // Configure Digital I/O Pin to be used as Chip Select for SPI
    // let cs = SysfsPin::new(26); // CE0, board J-24, GPIO 8 -> doesn work. use this from 2in19 example which works
    // cs.export().expect("cs export");
    // while !cs.is_exported() {}
    // cs.set_direction(Direction::Out).expect("CS Direction");
    // cs.set_value(1).expect("CS Value set to 1");
    //
    // Configure SPI
    // Settings are taken from

    println!(
        "Starting deskclock on device {}.",
        DeviceInfo::new()?.model()
    );

    println!("Setting up gpio ...");
    let gpio = Gpio::new()?;

    println!("Setting up busy pin ...");
    let busy = gpio.get(BUSY)?.into_input();

    println!("Setting up dc pin ...");
    let dc = gpio.get(DC)?.into_output();

    println!("Setting up rst pin ...");
    let rst = gpio.get(RST)?.into_output();

    println!("Opening spidev ...");
    // let mut spi = SpidevDevice::open("/dev/spidev0.0").expect("spidev directory");
    // let options = SpidevOptions::new()
    //     .bits_per_word(8)
    //     .max_speed_hz(10_000_000)
    //     .mode(spidev::SpiModeFlags::SPI_MODE_0)
    //     .build();
    // spi.configure(&options).expect("spi configuration");
    let mut spi = SimpleHalSpiDevice::new(Spi::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        10_000_000,
        Mode::Mode0,
    )?);

    let mut delay = Delay {};
    // Setup the epd
    println!("Setting up display ...");
    let mut epd = Epd2in7b::new(&mut spi, busy, dc, rst, &mut delay, None)?;

    // Setup the graphics
    println!("Setting up graphics ...");
    let mut display = Display2in7b::default();

    let character_style = MonoTextStyle::new(&FONT_6X10, Color::Black);
    // Draw some text
    println!("Drawing stuff ...");
    let _ = Text::new("Hello Rust!", Point::new(1, 1), character_style).draw(&mut display);

    // Transfer the frame data to the epd and display it
    println!("Updating display ...");
    epd.update_and_display_frame(&mut spi, &display.buffer(), &mut delay)?;

    Ok(())
}
