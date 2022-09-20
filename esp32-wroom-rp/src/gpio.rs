//! GPIO pin control interface of a connected ESP32-WROOM target Wifi board.
//!
//! ## Usage
//!
//! ```no_run
//! use esp32_wroom_rp::gpio::*;
//!
//! let mut pac = pac::Peripherals::take().unwrap();
//!
//! // The single-cycle I/O block controls our GPIO pins
//! let sio = hal::Sio::new(pac.SIO);
//!
//! // Set the pins to their default state
//! let pins = hal::gpio::Pins::new(
//!     pac.IO_BANK0,
//!     pac.PADS_BANK0,
//!     sio.gpio_bank0,
//!     &mut pac.RESETS,
//! );
//!
//! let esp_pins = esp32_wroom_rp::gpio::EspControlPins {
//!     // CS on pin x (GPIO7)
//!     cs: pins.gpio7.into_mode::<hal::gpio::PushPullOutput>(),
//!     // GPIO0 on pin x (GPIO2)
//!     gpio0: pins.gpio2.into_mode::<hal::gpio::PushPullOutput>(),
//!     // RESETn on pin x (GPIO11)
//!     resetn: pins.gpio11.into_mode::<hal::gpio::PushPullOutput>(),
//!     // ACK on pin x (GPIO10)
//!     ack: pins.gpio10.into_mode::<hal::gpio::FloatingInput>(),
//! };
//! ```

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{OutputPin, InputPin};

#[derive(Clone, Copy, Debug)]
pub enum IOError {
    Pin,
}

pub trait EspControlInterface {
    fn init(&mut self);

    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D);

    fn esp_select(&mut self);

    fn esp_deselect(&mut self);

    fn get_esp_ready(&self) -> bool;

    fn get_esp_ack(&self) -> bool;

    fn wait_for_esp_ready(&self);

    fn wait_for_esp_ack(&self);

    fn wait_for_esp_select(&mut self);
}

/// A structured representation of all GPIO pins that control a ESP32-WROOM NINA firmware-based
/// device outside of commands sent over the SPI/I²C bus. Pass a single instance of this struct
/// into `Wifi::init()`.
pub struct EspControlPins<CS: OutputPin, GPIO0: OutputPin, RESETN: OutputPin, ACK: InputPin> {
    pub cs: CS,
    pub gpio0: GPIO0,
    pub resetn: RESETN,
    pub ack: ACK,
}

impl<CS, GPIO0, RESETN, ACK> EspControlInterface for EspControlPins<CS, GPIO0, RESETN, ACK>
where
    CS: OutputPin,
    GPIO0: OutputPin,
    RESETN: OutputPin,
    ACK: InputPin,
{
    fn init(&mut self) {
        // Chip select is active-low, so we'll initialize it to a driven-high state
        self.cs.set_high().ok().unwrap();
        self.gpio0.set_high().ok().unwrap();
        self.resetn.set_high().ok().unwrap();
        self.get_esp_ready();
    }

    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.gpio0.set_high().ok().unwrap();
        self.cs.set_high().ok().unwrap();
        self.resetn.set_low().ok().unwrap();
        delay.delay_ms(10);
        self.resetn.set_high().ok().unwrap();
        delay.delay_ms(750);
    }

    fn esp_select(&mut self) {
        self.cs.set_low().ok().unwrap();
    }

    fn esp_deselect(&mut self) {
        self.cs.set_high().ok().unwrap();
    }

    fn get_esp_ready(&self) -> bool {
        self.ack.is_low().ok().unwrap()
    }

    fn get_esp_ack(&self) -> bool {
        self.ack.is_high().ok().unwrap()
    }

    fn wait_for_esp_ready(&self) {
        while self.get_esp_ready() != true {
            cortex_m::asm::nop(); // Make sure rustc doesn't optimize this loop out
        }
    }

    fn wait_for_esp_ack(&self) {
        while self.get_esp_ack() == false {
            cortex_m::asm::nop(); // Make sure rustc doesn't optimize this loop out
        }
    }

    fn wait_for_esp_select(&mut self) {
        self.wait_for_esp_ready();
        self.esp_select();
        self.wait_for_esp_ack();
    }
}

#[cfg(test)]
mod gpio_tests {
    use super::EspControlPins;
    use crate::gpio::EspControlInterface;
    use embedded_hal_mock::pin::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::MockError;
    use std::io::ErrorKind;

    #[test]
    fn gpio_init_sets_correct_state() {
        let err = MockError::Io(ErrorKind::NotConnected);

        let cs_expectations = [
            PinTransaction::set(PinState::High),
        ];

        let gpio0_expectations = [
            PinTransaction::set(PinState::High),
        ];

        let resetn_expectations = [
            PinTransaction::set(PinState::High),
        ];

        let ack_expectations = [
            PinTransaction::get(PinState::Low),
        ];

        let cs_mock = PinMock::new(&cs_expectations);
        let gpio0_mock = PinMock::new(&gpio0_expectations);
        let resetn_mock = PinMock::new(&resetn_expectations);
        let ack_mock = PinMock::new(&ack_expectations);
        let mut pins = EspControlPins {
            cs: cs_mock,
            gpio0: gpio0_mock,
            resetn: resetn_mock,
            ack: ack_mock,
        };

        pins.init();

        pins.cs.done();
        pins.gpio0.done();
        pins.resetn.done();
        pins.ack.done();
    }
}
