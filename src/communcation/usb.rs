use core::{cell::RefCell, sync::atomic::Ordering::Relaxed};

use atsamd_hal::{clock::GenericClockController, pac::Pm, usb::UsbBus};
use cortex_m::{interrupt::Mutex, singleton};
use atsamd_hal::pac::interrupt;
use defmt::warn;
use portable_atomic::AtomicUsize;
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

static MESSAGE: Mutex<RefCell<[u8; 64]>> = Mutex::new(RefCell::new([0; 64])); 
static MESSAGE_LENGTH: AtomicUsize = AtomicUsize::new(0);


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

                let message = &MESSAGE.borrow(cs).borrow();
                
                for &byte in &buf[..count] {
                    match byte {
                        b'\n' => {

                            let msg = core::str::from_utf8(&message[..MESSAGE_LENGTH.swap(0, Relaxed)]).unwrap_or("");

                            warn!("recieved {}", &msg);
                        }

                        _ => {
                            let length = &MESSAGE_LENGTH.load(Relaxed);
                            if length < &message.len() {
                                MESSAGE.borrow(cs).borrow_mut()[*length] = byte;
                                
                                MESSAGE_LENGTH.fetch_add(1, Relaxed);
                            } else {
                                MESSAGE_LENGTH.store(0, Relaxed);
                            }
                        },
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