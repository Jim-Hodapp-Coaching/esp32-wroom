//! esp32-wroom-rp
//! 
//! Rust-based Espressif ESP32-WROOM WiFi hardware abstraction layer for RP2040 series microcontroller.
//! Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//! 
//! NOTE This crate is still under active development. This API will remain volatile until 1.0.0


// This is just a placeholder for now. 
type Params = [u8; 5];

pub struct Wifi<C: NinaCommandHandler> {
  command_handler: C,
}

pub struct FirmwareVersion {
    major: u8,
    minor: u8,
    patch: u8
}

impl FirmwareVersion {
    fn new(version: [u8; 5]) -> FirmwareVersion {
        FirmwareVersion::parse(version)
    }

    // Takes in 5 bytes (e.g. 1.7.4) and returns a FirmwareVersion instance
    fn parse(version: [u8; 5]) -> FirmwareVersion {
        // TODO: real implementation
        FirmwareVersion {
            major: 1,
            minor: 7,
            patch: 4
        }
    }
}

impl<C: NinaCommandHandler> Wifi<C> {
    // fn connect(&self) -> Result<T> {
    //     self.command_handler.start_client_tcp()
    // }

    fn get_firmware_version(&self) -> Result<FirmwareVersion, Error> {
      self.command_handler.get_fw_version()
    }
}

impl<I: IoInterface> SpiCommandHandler<I> {
    fn send_command(command: u8, parameters: [u8; 5]) -> Result<FirmwareVersion, Error> {
        Ok(FirmwareVersion::new([0x31,0x2e,0x37,0x2e,0x34])) // 1.7.4
      }
}

impl<I: IoInterface> NinaCommandHandler for SpiCommandHandler<I> {

    const START_CLIENT_TCP: u8 = 0x2du8;
    const GET_FW_VERSION: u8 = 0x37u8;

    fn start_client_tcp(&self, params: Params) -> Result<FirmwareVersion, Error> {
        // TODO: implement a trait interface and set of structs for different parameter sets, e.g. SocketType
        SpiCommandHandler::send_command(self::START_CLIENT_TCP, params)
    }

    fn get_fw_version(&self) -> Result<FirmwareVersion, Error> {
        SpiCommandHandler::send_command(GET_FW_VERSION, [0; 5])
    }
}

struct SpiCommandHandler<I: IoInterface> {
    io_interface: I
}

trait NinaCommandHandler {
  const START_CLIENT_TCP: u8;
  const GET_FW_VERSION: u8;

  fn start_client_tcp(&self, params: Params) -> Result<FirmwareVersion, Error>;

  fn get_fw_version(&self) -> Result<FirmwareVersion, Error>;
}

trait IoInterface {

  fn esp_select(&mut self);

  fn esp_deselect(&mut self);

  fn get_esp_ready(&self) -> bool;

  fn get_esp_ack(&self) -> bool;

  fn wait_for_esp_ready(&self);

  fn wait_for_esp_ack(&self);

  fn wait_for_esp_select(&mut self);
  
}

struct IoInterfaceImpl {
  esp_pins: EspPins
}

impl IoInterface for IoInterfaceImpl {
    // TODO: add error handling
    fn esp_select(&mut self) {
        self.esp_pins.cs.set_low().unwrap();
    }

    fn esp_deselect(&mut self) {
        self.esp_pins.cs.set_high().unwrap();
    }

    fn get_esp_ready(&self) -> bool {
        self.esp_pins.ack.is_low().unwrap()
    }

    fn get_esp_ack(&self) -> bool {
        self.esp_pins.ack.is_high().unwrap()
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

struct EspPins {
    cs: Pin<Gpio7, hal::gpio::PushPullOutput>,
    gpio0: Pin<Gpio2, hal::gpio::PushPullOutput>,
    resetn: Pin<Gpio11, hal::gpio::PushPullOutput>,
    ack: Pin<Gpio10, hal::gpio::FloatingInput>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn firmware_parse_returns_a_populated_firmware_struct() {
        
    } 
}