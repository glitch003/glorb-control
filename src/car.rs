extern crate serialport;

use serialport::SerialPort;
use std::time::Duration;

use crate::sbus_writer::encode_sbus;

//  Define commands
#[derive(Debug)]
pub enum CarCommand {
    SendData(u16, u16, bool, bool),
    // Add other commands as needed
}

pub struct Car {
    serial_port: Box<dyn SerialPort>,
}

impl Car {
    pub fn new() -> Car {
        Car {
            serial_port: Self::init_serial(),
        }
    }

    fn init_serial() -> Box<dyn SerialPort> {
        // Define serial port parameters for SBUS
        let port_name = "/dev/tty.usbserial-ABSCDGUN"; // Adjust according to your OS and connected device
        let baud_rate = 100_000; // SBUS baud rate
        let timeout = Duration::from_millis(10);
        // Open the serial port with SBUS settings
        serialport::new(port_name, baud_rate)
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::Even)
            .stop_bits(serialport::StopBits::Two)
            .timeout(timeout)
            .open()
            .expect("Failed to open serial port")
    }

    pub fn send_data(&mut self, horizontal: u16, vertical: u16, forward: bool, armed: bool) {
        let mut channels: [u16; 16] = [1024; 16];
        channels[0] = horizontal;
        channels[2] = vertical;
        channels[6] = 240;
        channels[7] = 240;

        if forward {
            channels[5] = 1807;
        } else {
            channels[5] = 240;
        }

        if armed {
            channels[4] = 1807;
        } else {
            channels[4] = 240;
        }

        let packet = encode_sbus(channels);

        // println!("writing to serial port: {:?}", packet);

        self.serial_port
            .write(&packet)
            .expect("Serial write failed");
    }
}
