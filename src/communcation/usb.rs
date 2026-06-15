use core::{cell::RefCell, slice, sync::atomic::Ordering::Relaxed};

use atsamd_hal::{clock::GenericClockController, pac::Pm, usb::UsbBus};
use cortex_m::{interrupt::Mutex, singleton};
use atsamd_hal::pac::interrupt;
use defmt::warn;
use portable_atomic::{AtomicBool, AtomicUsize};
use usb_device::{LangID, bus::UsbBusAllocator, device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid}};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use crate::usb_allocator;

pub static USB_SERIAL: Mutex<RefCell<Option<SerialPort<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));
static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));

pub struct Usb; 

impl Usb {
    #[cfg(feature = "usb")]
    pub fn set_up(
        _clock: &mut GenericClockController,
        pm: &mut Pm,
        dm: impl Into<crate::UsbDm>,
        dp: impl Into<crate::UsbDp>,
        _usb: atsamd_hal::pac::Usb,
    ) 
    {
        cortex_m::interrupt::free(|cs| {

            
            let usb_alloc_ref = singleton!(: UsbBusAllocator<UsbBus> = usb_allocator(_usb, _clock, pm, dm, dp));
            let usb_alloc = usb_alloc_ref.unwrap();

            USB_SERIAL.borrow(cs).borrow_mut().replace(SerialPort::new(usb_alloc));

            USB_DEVICE.borrow(cs).borrow_mut().replace( UsbDeviceBuilder::new(usb_alloc, UsbVidPid(0x16c0, 0x27dd))
                    .strings(&[StringDescriptors::new(LangID::EN)
                        .manufacturer("GOO")
                        .product("grow one")])
                        .expect("Failed to set strings")
                    .device_class(USB_CLASS_CDC)
                    .self_powered(true)
                    .build());
        });
    }
}

static MESSAGE: Mutex<RefCell<[u8; 64]>> = Mutex::new(RefCell::new([0u8; 64])); 
static INDEX: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));
pub static ON: AtomicBool = AtomicBool::new(false);


fn poll_usb() {
    cortex_m::interrupt::free(|cs| {
        let mut serial_ref = USB_SERIAL.borrow(cs).borrow_mut();
        let serial = serial_ref.as_mut();
        
        let mut dev_ref = USB_DEVICE.borrow(cs).borrow_mut();
        let usb_device =  dev_ref.as_mut();

        if let (Some(device), Some(serial)) = (usb_device, serial) {
            if !device.poll(&mut [serial]) {
                return;
            }

            let mut buf = [0u8; 64];
            let mut msg = MESSAGE.borrow(cs).borrow_mut();
            let mut idx = INDEX.borrow(cs).borrow_mut();

            if let Ok(count) = serial.read(&mut buf) {
                for &byte in &buf[..count] {
                    serial.write(&[byte]).unwrap();
                    if *idx < msg.len() {
                       msg[*idx] = byte;
                        *idx += 1;
                    }

                    if byte == b'\n' || byte == b'\r' {
                        serial.write("\r\n".as_bytes()).unwrap();
                        if let Ok(s) = core::str::from_utf8(&msg[..*idx]) {
                            if s.trim() == "led" {
                                ON.fetch_xor(true, Relaxed);
                            }
                        }
                        *msg = [0u8; 64];
                        *idx = 0;
                    }
                }
            }

            // if let Ok(count) = serial.read(&mut buf) {
                
            //     for &byte in &buf[..count] {
            //         match byte {
            //             b'\n' => {
            //             }

            //             _ => {
            //                 MESSAGE.borrow(cs).
            //             },
            //         }
            //     }
            // };
        }
    });
}

#[interrupt]
fn USB() {
    poll_usb();
}