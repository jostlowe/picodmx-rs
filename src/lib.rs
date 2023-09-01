#![no_std]
#![no_main]

pub mod dmx_output;

/*
TODO: Finish comments and documentation
TODO: DMA fuckery
TODO: Get this shit on cargo
 */

/// The required clock frequency for the PIO assembly program
pub const PIO_CLOCK_FREQ: u32 = 500_000;
