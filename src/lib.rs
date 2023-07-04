#![no_std]
#![no_main]


use rp_pico as bsp;

use bsp::hal::clocks::{ClockSource, SystemClock};

use bsp::hal::pio::{
    PIOBuilder, PIOExt, PinDir, Running, ShiftDirection, StateMachine, StateMachineIndex, Tx,
    UninitStateMachine, PIO,
};
use bsp::hal::dma::{single_buffer::{Config, Transfer}, Channel, ChannelIndex};

/*
TODO: Finish comments and documentation
TODO: DMA fuckery
TODO: Get this shit on cargo
 */

/// The required clock frequency for the PIO assembly program
const PIO_CLOCK_FREQ: u32 = 500_000;

/// The main struct representing the DMX output hardware
pub struct Dmx<P: PIOExt, SM: StateMachineIndex> {
    sm: StateMachine<(P, SM), Running>,
    tx: Tx<(P, SM)>,
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

        let (mut stopped_sm, _rx, tx) = PIOBuilder::from_program(program)
            .out_pins(pin_id, 1)
            .side_set_pin_base(pin_id)
            .clock_divisor_fixed_point((system_clock_freq / PIO_CLOCK_FREQ) as u16, 0)
            .pull_threshold(8)
            .autopull(true)
            .out_shift_direction(ShiftDirection::Right)
            .build(sm);

        stopped_sm.set_pindirs([(pin_id, PinDir::Output)]);
        let sm = stopped_sm.start();

        Some(Self { sm, tx })
    }

    pub fn send_blocking(&mut self, start_code: u8, frame: &[u8]) {
        self.sm.restart();
        self.tx.write(start_code as u32);
        for channel in frame {
            while self.tx.is_full() {}
            self.tx.write(*channel as u32);
        }
        while !self.tx.has_stalled() {}
    }

    pub fn send<CH: ChannelIndex>(mut self, buf: &'static[u32], dma: Channel<CH>) -> DmxTransfer<P, SM, CH>{
        self.sm.restart();
        let transfer =  Config::new(dma, buf, self.tx).start();
        DmxTransfer{sm: self.sm, transfer}
    }
}

pub struct DmxTransfer<P: PIOExt, SM: StateMachineIndex, CH: ChannelIndex> {
    sm: StateMachine<(P, SM), Running>,
    transfer: Transfer<Channel<CH>, &'static[u32], Tx<(P, SM)>>,
}

impl<P, SM, CH> DmxTransfer<P, SM, CH>
    where
        P: PIOExt,
        SM: StateMachineIndex,
        CH: ChannelIndex
{
    pub fn busy(&self) -> bool {
        !self.sm.stalled()
    }

    pub fn wait(self) -> (Dmx<P, SM>, Channel<CH>) {
        while self.busy() {}
        let (ch, _buf, tx) = self.transfer.wait();
        (Dmx{sm: self.sm, tx}, ch)
    }
}
