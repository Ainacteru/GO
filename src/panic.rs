use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // let info = info.message();
    // let info = info.as_str().unwrap();

    // let mut peripherals = unsafe { Peripherals::steal() };
    // let core = unsafe { CorePeripherals::steal() };
    // let mut clocks = GenericClockController::with_external_32kosc(
    //     peripherals.gclk,
    //     &mut peripherals.pm,
    //     &mut peripherals.sysctrl,
    //     &mut peripherals.nvmctrl,
    // );

   

    // let mut delay = Delay::new(core.SYST, &mut clocks);
    // let pins = crate::Pins::new(peripherals.port);
    // let mut red:RgbRed = pins.rgb_red.into();

    loop {
        defmt::error!("panic! uhoh{}", defmt::Display2Format(info));

        cortex_m::asm::delay(5_000_000);
        
    }
}