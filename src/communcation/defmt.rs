use core::{cell::RefCell, sync::atomic::{AtomicBool}};
use cortex_m::interrupt::{Mutex, free};
use defmt::Encoder;
use portable_atomic::{Ordering};

use crate::communcation::usb::USB_SERIAL;


static USB_LOGGER: UsbLogger = UsbLogger::new();

#[defmt::global_logger]
struct Logger;

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        cortex_m::interrupt::disable();
        USB_LOGGER.acquire();
    }

    unsafe fn flush() {
        USB_LOGGER.flush(); 
    }

    unsafe fn release() {
       USB_LOGGER.release();
        cortex_m::interrupt::enable();
    }

    unsafe fn write(bytes: &[u8]) {
        USB_LOGGER.write(bytes);
    }
}

struct UsbLogger {
    taken: AtomicBool, 
    encoder: Mutex<RefCell<Encoder>>,
}

impl UsbLogger {
    const fn new() -> Self {
        Self {
            taken: AtomicBool::new(false),
            encoder: Mutex::new(RefCell::new(Encoder::new())),
        }
    }

    fn acquire(&self) {
        if self.taken.load(Ordering::Relaxed) {
            panic!("defmt logger re-entered");
        }

        self.taken.store(true, Ordering::Relaxed);

        free(|cs| {
            self.encoder.borrow(cs).borrow_mut().start_frame(UsbWriter::write_byte);
        });
    }

    unsafe fn flush(&self) {
        
    }

    unsafe fn release(&self) {
        free(|cs| {
            self.encoder.borrow(cs).borrow_mut().end_frame(UsbWriter::write_byte);
        });
        self.taken.store(false, Ordering::Relaxed);
    }

    unsafe fn write(&self, bytes: &[u8]) {
        free(|cs| {
            self.encoder.borrow(cs).borrow_mut().write(bytes, UsbWriter::write_byte);
        });
    }
}

pub struct UsbWriter;

impl UsbWriter {
    pub fn write_byte(bytes: &[u8]) {
        free(|cs| {
            let mut serial = USB_SERIAL.borrow(cs).borrow_mut();

            if let Some(serial) = serial.as_mut() {
                let _ = serial.write(bytes);
            }
        });
    }
}