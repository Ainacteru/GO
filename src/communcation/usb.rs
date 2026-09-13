use core::{cell::RefCell};

use atsamd_hal::{clock::GenericClockController, pac::{Pm, SCB}, usb::UsbBus};
use cortex_m::{interrupt::Mutex, singleton};
use atsamd_hal::pac::interrupt;
use usb_device::{LangID, bus::UsbBusAllocator, device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid}};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use crate::communcation::command_parser::{CommandParser, CommandSource};

pub static USB_SERIAL: Mutex<RefCell<Option<SerialPort<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));
static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<'static, UsbBus>>>> = Mutex::new(RefCell::new(None));

static USB_PARSER: Mutex<RefCell<CommandParser>> = Mutex::new(RefCell::new(CommandParser::new()));

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
            use crate::usb_allocator;


            
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
        let mut dev_ref = USB_DEVICE.borrow(cs).borrow_mut();

        let Some(serial) = serial_ref.as_mut() else {
            return
        };

        let Some(device) = dev_ref.as_mut() else {
            return
        };

        if !device.poll(&mut [serial]) {
            return;
        }

        let mut buf = [0u8; 64];

       if let Ok(count) = serial.read(&mut buf) {
            for &byte in &buf[..count] {
                serial.write(&[byte]).ok();
                feed_usb(byte);
            }
        }
    });
}

fn feed_usb(byte: u8) {
    let command = cortex_m::interrupt::free(|cs| {
        USB_PARSER
            .borrow(cs)
            .borrow_mut()
            .push(byte)
    });

    if let Some(command) = command {
        CommandParser::handle_command(CommandSource::Usb, command);
    }
}



#[interrupt]
fn USB() {
    poll_usb();
}