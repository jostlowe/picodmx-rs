#![no_std]
#![no_main]

use picodmx::Dmx;

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{FunctionPio0, Pin},
    pac,
    pio::PIOExt,
    sio::Sio,
    watchdog::Watchdog,
};
use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {

    // Do all of our regular setup
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Pick a pin for our DMX output and change it to the proper mode
    let dmx_pin: Pin<_, FunctionPio0> = pins.gpio0.into_mode();

    // Get its `pin_id` for later use
    let dmx_pin_id = dmx_pin.id().num;

    // Get a PIO and a state machine for use in the DMX output
    let (mut pio, sm, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    // Create the DMX output
    let mut dmx = Dmx::new(&mut pio, sm, dmx_pin_id, &clocks.system_clock).unwrap();

    // Create a led pin that we can blink when the output is working
    let mut led_pin = pins.led.into_push_pull_output();

    loop {
        led_pin.set_high().unwrap();

        // Send our data woooshing down the line with a 0x00 start code
        dmx.send(0x00, &[0xff, 0xaa, 0xff, 0xaa]);
        led_pin.set_low().unwrap();
        delay.delay_ms(50)
    }
}
