use core::{cell::RefCell};

use atsamd_hal::{clock::GenericClockController, pac::Pm, usb::UsbBus};
use cortex_m::{interrupt::Mutex, singleton};
use atsamd_hal::pac::interrupt;
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
                    .build());
        });
    }
}

fn poll_usb() {
    cortex_m::interrupt::free(|cs| {
        let mut serial_ref = USB_SERIAL.borrow(cs).borrow_mut();
        let serial = serial_ref.as_mut();
        
        let mut dev_ref = USB_DEVICE.borrow(cs).borrow_mut();
        let usb_device =  dev_ref.as_mut();

        if let (Some(_device), Some(serial)) = (&usb_device, serial) {
            usb_device.unwrap().poll(&mut [serial]);

            let mut buf = [0u8; 64];

            if let Ok(count) = serial.read(&mut buf) {
                for (i, c) in buf.iter().enumerate() {
                    let c = *c as char;
                    if i >= count {
                        break;
                    }
                    match c {
                        '\r' => { serial.write(b"\r\n").ok(); }
                        _ => { serial.write(&[c as u8]).ok(); }
                    }
                }
            };
        }
    });
}

#[interrupt]
fn USB() {
    poll_usb();
}