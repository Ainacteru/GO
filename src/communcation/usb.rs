use core::cell::RefCell;
use heapless::String;
use cortex_m::interrupt::Mutex;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usb_device::device::UsbDeviceState;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use crate::hal::clock::GenericClockController;
use crate::hal::usb::UsbBus;
use crate::pac::{self, interrupt};
use crate::{usb_allocator, UsbDm, UsbDp};
use cortex_m::peripheral::NVIC;
use crate::{uprint, uprintln};

static USB_ALLOCATOR: Mutex<RefCell<Option<UsbBusAllocator<UsbBus>>>> = Mutex::new(RefCell::new(None));
static USB_BUS: Mutex<RefCell<Option<UsbDevice<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));
pub static USB_SERIAL: Mutex<RefCell<Option<SerialPort<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));

static MESSAGE: Mutex<RefCell<String<64>>> = Mutex::new(RefCell::new(String::new()));
static USB_INPUT_READY: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

pub fn set_input_ready() {
    cortex_m::interrupt::free(|cs| {
        *USB_INPUT_READY.borrow(cs).borrow_mut() = true;
    });
}

pub fn set_up(
    usb: pac::Usb,
    clocks: &mut GenericClockController,
    pm: &mut pac::Pm,
    usb_dm: impl Into<UsbDm>,
    usb_dp: impl Into<UsbDp>,
    nvic: &mut NVIC,
) {
    cortex_m::interrupt::free(|cs| {
        USB_ALLOCATOR.borrow(cs).replace(Some(usb_allocator(usb, clocks, pm, usb_dm, usb_dp)));
    });

    let allocator: &'static UsbBusAllocator<UsbBus> = unsafe {
        cortex_m::interrupt::free(|cs| {
            &*(USB_ALLOCATOR.borrow(cs).borrow().as_ref().unwrap() as *const _)
        })
    };

    cortex_m::interrupt::free(|cs| {
        USB_SERIAL.borrow(cs).replace(Some(SerialPort::new(allocator)));
        USB_BUS.borrow(cs).replace(Some(
            UsbDeviceBuilder::new(allocator, UsbVidPid(0x16c0, 0x27dd))
                .strings(&[StringDescriptors::new(LangID::EN)
                    .manufacturer("ARI IS GROSS HAHAHAHAAH")
                    .product("Serial port")
                    .serial_number("TEST")])
                .expect("Failed to set strings")
                .device_class(USB_CLASS_CDC)
                .build(),
        ));
    });

    unsafe {
        nvic.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }
}

fn poll_usb() {
    let mut message_ready = false;
    let mut buffer_full = false;
    let mut echo_buf = String::<64>::new();

    cortex_m::interrupt::free(|cs| {
        let mut bus_ref = USB_BUS.borrow(cs).borrow_mut();
        let mut serial_ref = USB_SERIAL.borrow(cs).borrow_mut();
        let (Some(usb_dev), Some(serial)) = (bus_ref.as_mut(), serial_ref.as_mut()) else {
            return;
        };

        usb_dev.poll(&mut [serial as &mut dyn usb_device::class::UsbClass<UsbBus>]);

        if usb_dev.state() != UsbDeviceState::Configured {
            return;
        }
        if !*USB_INPUT_READY.borrow(cs).borrow() {
            let mut d = [0u8; 64];
            serial.read(&mut d).ok();
            return;
        }

        let mut buf = [0u8; 64];
        if let Ok(count) = serial.read(&mut buf) {
            if let Ok(s) = core::str::from_utf8(&buf[..count]) {
                for c in s.chars() {
                    if c == '\r' {
                        message_ready = true;
                    } else if c == '\x08' || c == '\x7f' {
                        let mut msg = MESSAGE.borrow(cs).borrow_mut();
                        if msg.pop().is_some() {
                            echo_buf.push('\x08').ok();
                            echo_buf.push(' ').ok();
                            echo_buf.push('\x08').ok();
                        }
                    } else {
                        let mut msg = MESSAGE.borrow(cs).borrow_mut();
                        echo_buf.push(c).ok();
                        if msg.push(c).is_err() {
                            buffer_full = true;
                            msg.clear();
                        }
                    }
                }
            }
        }
    });

    if !echo_buf.is_empty() {
        uprint!("{}", echo_buf.as_str());
    }
    if buffer_full {
        uprintln!("\nBuffer full!");
    }
    if message_ready {
        uprintln!("");
        cortex_m::interrupt::free(|cs| {
            let msg = MESSAGE.borrow(cs).borrow().clone();
            handle_message(&msg);
            MESSAGE.borrow(cs).borrow_mut().clear();
        });
    }
}

fn handle_message(msg: &str) {
    uprintln!("Received: {}", msg);
}

#[interrupt]
fn USB() {
    poll_usb();
}

pub struct UsbWriter;
impl core::fmt::Write for UsbWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        cortex_m::interrupt::free(|cs| {
            if let Some(serial) = USB_SERIAL.borrow(cs).borrow_mut().as_mut() {
                serial.write(s.trim_end_matches('\n').as_bytes()).ok();
            }
        });
        Ok(())
    }
}

#[macro_export]
macro_rules! uprintln {
    ($($arg:tt)*) => {{
        cortex_m::interrupt::free(|cs| {
            use core::fmt::Write;
            core::write!($crate::communcation::usb::UsbWriter, $($arg)*).ok();
            if let Some(s) = $crate::communcation::usb::USB_SERIAL.borrow(cs).borrow_mut().as_mut() {
                s.write(b"\r\n").ok();
            }
        });
    }};
}

#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        core::write!($crate::communcation::usb::UsbWriter, $($arg)*).ok();
    }};
}
