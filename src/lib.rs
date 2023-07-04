#![no_std]
#![no_main]

use rp_pico as bsp;

use bsp::hal::clocks::{ClockSource, SystemClock};

use bsp::hal::pio::{
    PIOBuilder, PIOExt, PinDir, Running, Rx, ShiftDirection, StateMachine, StateMachineIndex, Tx,
    UninitStateMachine, PIO,
};

/*
TODO: Make assembly code "inline" or fix the macro shite.
TODO: Finish comments and documentation
TODO: Split DMX into a two separate components. One API and one HW
TODO: Make an async version using a DMA channel and HW
TODO: Get this shit on cargo
 */

/// The required clock frequency for the PIO assembly program
const PIO_CLOCK_FREQ: u32 = 500_000;

/// The main struct representing the DMX output hardware
pub struct Dmx<P: PIOExt, SM: StateMachineIndex> {
    /// The PIO state machine we are using for DMX output
    sm: StateMachine<(P, SM), Running>,

    /// The TX-queue where we push our outgoing data frame
    tx: Tx<(P, SM)>,

    /// The RX-queue is not used, but is not discarded, as it is needed to uninit
    /// the state machine when we end the DMX output
    rx: Rx<(P, SM)>,
}

impl<P, SM> Dmx<P, SM>
    where
        P: PIOExt,
        SM: StateMachineIndex,
{
    /// Create a new DMX instance. Returns `None` if there is not enough room to install the DMX
    /// output program in the PIO
    pub fn new(
        pio: &mut PIO<P>,
        sm: UninitStateMachine<(P, SM)>,
        pin_id: u8,
        system_clock: &SystemClock,
    ) -> Option<Self> {
        let uninstalled_program = pio_proc::pio_file!("src/dmx_output.pio").program;
        let program = pio.install(&uninstalled_program).ok()?;
        let system_clock_freq = system_clock.get_freq().to_Hz();

        let (mut stopped_sm, rx, tx) = PIOBuilder::from_program(program)
            .out_pins(pin_id, 1)
            .side_set_pin_base(pin_id)
            .clock_divisor_fixed_point((system_clock_freq / PIO_CLOCK_FREQ) as u16, 0)
            .pull_threshold(8)
            .autopull(true)
            .out_shift_direction(ShiftDirection::Right)
            .build(sm);

        stopped_sm.set_pindirs([(pin_id, PinDir::Output)]);
        let sm = stopped_sm.start();

        Some(Self { sm, tx, rx })
    }

    pub fn send(&mut self, start_code: u8, frame: &[u8]) {
        self.sm.restart();
        self.tx.write(start_code as u32);
        for channel in frame {
            while self.tx.is_full() {}
            self.tx.write(*channel as u32);
        }
        while !self.tx.has_stalled() {}
    }

    pub fn end(self) -> UninitStateMachine<(P, SM)> {
        self.sm.stop().uninit(self.rx, self.tx).0
    }
}
