#![no_std]


pub mod communcation;
pub mod actuators;

#[cfg(feature = "rt")]
pub use cortex_m_rt::entry;

pub use atsamd_hal as hal;
pub use hal::ehal;
pub use hal::pac;

use hal::clock::GenericClockController;
use hal::sercom::{i2c, spi};
use hal::time::Hertz;

#[cfg(feature = "usb")]
use hal::usb::{usb_device::bus::UsbBusAllocator, UsbBus};

hal::bsp_peripherals!(
    Sercom3 { I2cSercom }
    Sercom4 { SpiSercom }
);

/// Definitions related to pins and pin aliases
pub mod pins {
    use super::hal;

    hal::bsp_pins!(
        PA02 {
            /// Pyro channel 2 output
            name: pyro2
            aliases: {
                PushPullOutput: Pyro2
            }
        }
        PA03 {
            /// AREF pin - has 1uF capacitor to ground
            name: aref
        }
        PA04 {
            /// RGB LED blue channel
            name: rgb_blue
            aliases: {
                PushPullOutput: RgbBlue
                AlternateE: RgbBluePwm

            }
        }
        PA05 {
            /// RGB LED green channel
            name: rgb_green
            aliases: {
                PushPullOutput: RgbGreen
                AlternateE: RgbGreenPwm
            }
        }
        PA06 {
            /// SPI chip select for external flash
            name: flash_cs
            aliases: {
                PushPullOutput: FlashCs
            }
        }
        PA07 {
            /// RF module antenna switch
            name: rf_sw
            aliases: {
                PushPullOutput: RfSw
            }
        }
        PA08 {
            /// RF module interrupt
            name: rf_int
            aliases: {
                PullUpInput: RfInt
            }
        }
        PA09 {
            /// RF module busy signal
            name: rf_busy
            aliases: {
                PullUpInput: RfBusy
            }
        }
        PA10 {
            /// RF module chip select
            name: rf_cs
            aliases: {
                PushPullOutput: RfCs
            }
        }
        PA11 {
            /// SD card detect
            name: sd_detect
            aliases: {
                PullUpInput: SdDetect
            }
        }
        PA12 {
            /// SPI MISO
            name: miso
            aliases: {
                AlternateD: Miso
            }
        }
        PA13 {
            /// Buzzer output
            name: buzzer
            aliases: {
                PushPullOutput: Buzzer
            }
        }
        PA14 {
            /// SD card chip select
            name: sd_cs
            aliases: {
                PushPullOutput: SdCs
            }
        }
        PA16 {
            /// Servo 1 PWM output
            name: servo1
            aliases: {
                AlternateF: Servo1Pwm
            }
        }
        PA17 {
            /// Status LED
            name: led
            aliases: {
                PushPullOutput: Led
            }
        }
        PA18 {
            /// Servo 2 PWM output
            name: servo2
            aliases: {
                AlternateF: Servo2Pwm
            }
        }
        PA19 {
            /// Servo 3 PWM output
            name: servo3
            aliases: {
                AlternateF: Servo3Pwm
            }
        }
        PA20 {
            /// External flash enable
            name: flash_en
            aliases: {
                PushPullOutput: FlashEn
            }
        }
        PA21 {
            /// RGB LED red channel
            name: rgb_red
            aliases: {
                PushPullOutput: RgbRed
                AlternateF: RgbRedPwm
            }
        }
        PA22 {
            /// I2C data line
            name: sda
            aliases: {
                AlternateC: Sda
            }
        }
        PA23 {
            /// I2C clock line
            name: scl
            aliases: {
                AlternateC: Scl
            }
        }
        PA24 {
            /// USB D-
            name: usb_dm
            aliases: {
                AlternateG: UsbDm
            }
        }
        PA25 {
            /// USB D+
            name: usb_dp
            aliases: {
                AlternateG: UsbDp
            }
        }
        PB02 {
            /// Pyro channel 1 output
            name: pyro1
            aliases: {
                PushPullOutput: Pyro1
            }
        }
        PB08 {
            /// RF module reset
            name: rf_nrst
            aliases: {
                PushPullOutput: RfNrst
            }
        }
        PB10 {
            /// SPI MOSI
            name: mosi
            aliases: {
                AlternateD: Mosi
            }
        }
        PB11 {
            /// SPI SCLK
            name: sclk
            aliases: {
                AlternateD: Sclk
            }
        }
    );
}
pub use pins::*;

/// SPI pads for the labelled SPI peripheral
pub type SpiPads = spi::Pads<SpiSercom, Miso, Mosi, Sclk>;

/// SPI master for the labelled SPI peripheral
pub type Spi = spi::Spi<spi::Config<SpiPads>, spi::Duplex>;

/// Convenience for setting up the SPI peripheral (Sercom4, SPI Mode 0).
pub fn spi_master(
    clocks: &mut GenericClockController,
    baud: Hertz,
    sercom: SpiSercom,
    pm: &mut pac::Pm,
    sclk: impl Into<Sclk>,
    mosi: impl Into<Mosi>,
    miso: impl Into<Miso>,
) -> Spi {
    let gclk0 = clocks.gclk0();
    let clock = clocks.sercom4_core(&gclk0).unwrap();
    let freq = clock.freq();
    let (miso, mosi, sclk) = (miso.into(), mosi.into(), sclk.into());
    let pads = spi::Pads::default().data_in(miso).data_out(mosi).sclk(sclk);
    spi::Config::new(pm, sercom, pads, freq)
        .baud(baud)
        .spi_mode(spi::MODE_0)
        .enable()
}

/// I2C pads for the labelled I2C peripheral
pub type I2cPads = i2c::Pads<I2cSercom, Sda, Scl>;

/// I2C master for the labelled I2C peripheral
pub type I2c = i2c::I2c<i2c::Config<I2cPads>>;

/// Convenience for setting up the I2C peripheral (Sercom3).
pub fn i2c_master(
    clocks: &mut GenericClockController,
    baud: impl Into<Hertz>,
    sercom: I2cSercom,
    pm: &mut pac::Pm,
    sda: impl Into<Sda>,
    scl: impl Into<Scl>,
) -> I2c {
    let gclk0 = clocks.gclk0();
    let clock = &clocks.sercom3_core(&gclk0).unwrap();
    let freq = clock.freq();
    let baud = baud.into();
    let pads = i2c::Pads::new(sda.into(), scl.into());
    i2c::Config::new(pm, sercom, pads, freq).baud(baud).enable()
}

#[cfg(feature = "usb")]
/// Convenience function for setting up USB
pub fn usb_allocator(
    usb: pac::Usb,
    clocks: &mut GenericClockController,
    pm: &mut pac::Pm,
    dm: impl Into<UsbDm>,
    dp: impl Into<UsbDp>,
) -> UsbBusAllocator<UsbBus> {
    let gclk0 = clocks.gclk0();
    let clock = &clocks.usb(&gclk0).unwrap();
    let (dm, dp) = (dm.into(), dp.into());
    UsbBusAllocator::new(UsbBus::new(clock, pm, dm, dp, usb))
}
