use rp_pico as bsp;

use bsp::hal::clocks::{ClockSource, SystemClock};

use bsp::hal::pio::{
    PIOBuilder, PIOExt, PinDir, Running, Rx, ShiftDirection, StateMachine, StateMachineIndex,
    UninitStateMachine, PIO,
};

use crate::PIO_CLOCK_FREQ;

/// The main struct representing the DMX input hardware
pub struct DmxInput<P: PIOExt, SM: StateMachineIndex> {
    pub sm: StateMachine<(P, SM), Running>,
    pub rx: Rx<(P, SM)>,
}

impl<P, SM> DmxInput<P, SM>
where
    P: PIOExt,
    SM: StateMachineIndex,
{
    /// Create a new DMX instance. Returns `None` if there is not enough room to install the DMX
    /// input program in the PIO
    pub fn new(
        pio: &mut PIO<P>,
        sm: UninitStateMachine<(P, SM)>,
        pin_id: u8,
        system_clock: &SystemClock,
    ) -> Option<Self> {
        let uninstalled_program = pio_proc::pio_file!("src/dmx_input.pio").program;
        let program = pio.install(&uninstalled_program).ok()?;
        let system_clock_freq = system_clock.get_freq().to_Hz();

        let (mut stopped_sm, rx, _tx) = PIOBuilder::from_program(program)
            .in_pin_base(pin_id)
            .clock_divisor_fixed_point((system_clock_freq / PIO_CLOCK_FREQ) as u16, 0)
            .push_threshold(8)
            .autopush(true)
            .in_shift_direction(ShiftDirection::Right)
            .build(sm);

        stopped_sm.set_pindirs([(pin_id, PinDir::Input)]);
        let sm = stopped_sm.start();

        Some(Self { sm, rx })
    }

    pub fn read_blocking(&mut self, buf: &mut [u8]) {
        let mut buf = buf.iter_mut();
        self.sm.restart();
        while self.rx.is_empty() {}
        if let Some(channels) = self.rx.read() {
            for channel in channels.to_be_bytes() {
                if let Some(buf_channel) = buf.next() {
                    *buf_channel = channel;
                }
            }
        }
    }
}
