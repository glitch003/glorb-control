// extern crate hidapi;
extern crate serialport;

use std::io::Write;
use std::time::Duration;

mod sbus_parser;
use joycon_rs::prelude::input_report_mode::PushedButtons;
use sbus_parser::SBusPacketParser;

mod sbus_writer;
use sbus_writer::encode_sbus;

// use hidapi::HidApi;
use joycon_rs::prelude::*;
use serialport::SerialPort;

use std::collections::HashMap;

fn map_range(value: u16, from_range: (u16, u16), to_range: (u16, u16)) -> u16 {
    let (from_min, from_max) = from_range;
    let (to_min, to_max) = to_range;

    if from_min == from_max {
        return ((to_min as u32 + to_max as u32) / 2) as u16; // Midpoint of to_range
    }

    // Ensure the value is within the from_range
    let value = if value > from_max {
        from_max
    } else if value < from_min {
        from_min
    } else {
        value
    };

    // Linearly interpolate the value between the source range
    let proportion =
        (value as u32 - from_min as u32) as f64 / (from_max as u32 - from_min as u32) as f64;

    // Map the proportion to the destination range
    let result = (to_min as f64 + proportion * (to_max - to_min) as f64).round() as u32;

    // Ensure the result is within the u16 range
    result.min(65535) as u16
}

// fn process_report(report: &[u8; 256]) -> HashMap<&'static str, u8> {
//     let mut result = HashMap::new();

//     result.insert("up", if report[1] < 128 { 128 - report[1] } else { 0 });
//     result.insert("down", if report[1] > 128 { report[1] - 127 } else { 0 });
//     result.insert("right", if report[2] < 128 { 128 - report[2] } else { 0 });
//     result.insert("left", if report[2] > 128 { report[2] - 127 } else { 0 });

//     result
// }

fn process_report(report: &[u8; 256]) -> HashMap<&'static str, u8> {
    let mut result = HashMap::new();

    // result.insert("up", if report[8] < 128 { 128 - report[8] } else { 0 });
    // result.insert("down", if report[8] > 128 { report[8] - 127 } else { 0 });
    // result.insert("right", if report[7] < 128 { 128 - report[7] } else { 0 });
    // result.insert("left", if report[7] > 128 { report[7] - 127 } else { 0 });

    result.insert("right/left", report[7]);
    result.insert("up/down", report[8]);

    result
}

fn send_data_to_car(
    horizontal: u16,
    vertical: u16,
    forward: bool,
    armed: bool,
    serial_port: &mut dyn SerialPort,
) {
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

    serial_port.write(&packet);
}

fn remap_left_joycon(horizontal: u16, vertical: u16) -> (u16, u16) {
    // horizontal min (left) 670
    // horizotal max (right) 3420
    let horizontal_mapped = map_range(horizontal, (670, 3240), (240, 1807));

    // vertical min (down) 1080
    // vertical max (up) 3240
    let vertical_mapped = map_range(vertical, (1080, 3240), (240, 1807));

    (horizontal_mapped, vertical_mapped)
}

fn remap_right_joycon(horizontal: u16, vertical: u16) -> (u16, u16) {
    // horizontal min (left) 700
    // horizotal max (right) 3600
    let horizontal_mapped = map_range(horizontal, (700, 3600), (240, 1807));

    // vertical min (down) 780
    // vertical max (up) 3000
    let vertical_mapped = map_range(vertical, (780, 3000), (240, 1807));

    (horizontal_mapped, vertical_mapped)
}

fn main() {
    let manager = JoyConManager::get_instance();
    let (managed_devices, new_devices) = {
        let lock = manager.lock();
        match lock {
            Ok(m) => (m.managed_devices(), m.new_devices()),
            Err(_) => unreachable!(),
        }
    };

    managed_devices
        .into_iter()
        .chain(new_devices)
        .flat_map(|dev| SimpleJoyConDriver::new(&dev))
        .try_for_each::<_, JoyConResult<()>>(|driver| {
            println!("Found JoyCon: {:?}", driver.joycon().device_type());

            // Change JoyCon to Simple hid mode.
            // let simple_hid_mode = SimpleHIDMode::new(driver)?;
            let standard_full_mode = StandardFullMode::new(driver)?;

            // Spawn thread
            std::thread::spawn(move || {
                let mut forward = true;
                let mut armed = false;
                // Define serial port parameters for SBUS
                let port_name = "/dev/tty.usbserial-ABSCDGUN"; // Adjust according to your OS and connected device
                let baud_rate = 100_000; // SBUS baud rate
                let timeout = Duration::from_millis(10);
                // Open the serial port with SBUS settings
                let mut port = serialport::new(port_name, baud_rate)
                    .data_bits(serialport::DataBits::Eight)
                    .parity(serialport::Parity::Even)
                    .stop_bits(serialport::StopBits::Two)
                    .timeout(timeout)
                    .open()
                    .expect("Failed to open serial port");
                loop {
                    // Forward the report to the main thread
                    // println!("{:?}", simple_hid_mode.read_input_report());
                    // println!("{:?}", standard_full_mode.read_input_report());
                    let report = standard_full_mode.read_input_report();
                    match report {
                        Ok(report) => {
                            // println!("{:?}", report);
                            // let horizontal = report.common.left_analog_stick_data.horizontal;

                            let (horizontal_mapped, vertical_mapped) = remap_left_joycon(
                                report.common.left_analog_stick_data.horizontal,
                                report.common.left_analog_stick_data.vertical,
                            );
                            // println!(
                            //     "unmapped\tHorizontal: {}\tVertical: {}",
                            //     report.common.right_analog_stick_data.horizontal,
                            //     report.common.right_analog_stick_data.vertical
                            // );
                            println!(
                                "mapped\tHorizontal: {}\tVertical: {}, forward: {}, armed: {}",
                                horizontal_mapped, vertical_mapped, forward, armed
                            );
                            if report.common.pushed_buttons.contains(Buttons::Down) {
                                println!("Down pressed");
                                forward = false;
                            } else if report.common.pushed_buttons.contains(Buttons::Up) {
                                println!("Down pressed");
                                forward = true;
                            } else if report.common.pushed_buttons.contains(Buttons::Left)
                                && report.common.pushed_buttons.contains(Buttons::Right)
                            {
                                println!("Left and right pressed - armed");
                                armed = true;
                            } else if report.common.pushed_buttons.contains(Buttons::SL)
                                && report.common.pushed_buttons.contains(Buttons::SR)
                            {
                                println!("L and R pressed - unarmed");
                                armed = false;
                            }
                            send_data_to_car(
                                horizontal_mapped,
                                vertical_mapped,
                                forward,
                                armed,
                                &mut *port,
                            );
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            });

            Ok(())
        })
        .unwrap();

    // println!("Printing all available hid devices:");
    // match HidApi::new() {
    //     Ok(api) => {
    //         for device in api.device_list() {
    //             println!(
    //                 "{} - {:04x}:{:04x}",
    //                 device.product_string().unwrap_or(""),
    //                 device.vendor_id(),
    //                 device.product_id()
    //             );
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("Error: {}", e);
    //     }
    // }

    // // let gamepad_id = [0x045e, 0x0040];
    // let gamepad_id = [0x057e, 0x2006];
    // let api = HidApi::new().unwrap();
    // let device = api.open(gamepad_id[0], gamepad_id[1]).unwrap();

    // loop {
    //     let mut buf = [0u8; 256];
    //     let res = device.read(&mut buf[..]).unwrap();
    //     println!("Read: {:?}", &buf[..res]);
    //     let report = process_report(&buf);
    //     // println!("{:?}", report);
    // }

    // // good packet 0ff000200001f8f087c3031e00042000010840000210800000
    // let mut channels: [u16; 16] = [1024; 16];
    // channels[0] = 240;
    // channels[4] = 1807;
    // channels[5] = 1807;
    // channels[6] = 240;
    // channels[7] = 240;

    // let packet = encode_sbus(channels);
    // println!("0ff000200001f8f087c3031e00042000010840000210800000");
    // println!(
    //     "{}",
    //     packet
    //         .iter()
    //         .map(|&v| format!("{:02X}", v))
    //         .collect::<Vec<String>>()
    //         .join("")
    // );

    // // parse it
    // let mut parser = SBusPacketParser::new();
    // parser.push_bytes(&packet);
    // let parsed_packet = parser.try_parse().unwrap();
    // println!("{:?}", parsed_packet);
}
