use std::{sync::mpsc, thread};

use joycon_rs::prelude::*;

// mod sbus_parser;
// use sbus_parser::SBusPacketParser;

mod sbus_writer;

mod car;
#[cfg(feature = "car")]
use car::Car;
use car::CarCommand;

mod utils;

mod joycons;
use joycons::remap_left_joycon;

use crate::joycons::JoyConState;

fn main() {
    // Create a channel for sending commands
    let (tx, rx) = mpsc::channel();

    //  Spawn a dedicated thread that owns `car`
    let _handle = thread::spawn(move || {
        #[cfg(feature = "car")]
        let mut car = Car::new();
        for command in rx {
            // println!("Sending command to car: {:?}", command);
            match command {
                CarCommand::SendData(horizontal_mapped, vertical_mapped, forward, armed) => {
                    #[cfg(feature = "car")]
                    car.send_data(horizontal_mapped, vertical_mapped, forward, armed);
                } // Handle other commands as needed
            }
        }
    });

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

            let tx_clone = tx.clone();

            // Change JoyCon to Simple hid mode.
            // let simple_hid_mode = SimpleHIDMode::new(driver)?;
            let standard_full_mode = StandardFullMode::new(driver)?;

            // Spawn thread
            thread::spawn(move || {
                let mut joycon_states = vec![
                    JoyConState {
                        forward: true,
                        armed: false,
                    },
                    JoyConState {
                        forward: true,
                        armed: false,
                    },
                ];

                loop {
                    // Forward the report to the car thread
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

                            // left joycon buttons
                            if report.common.pushed_buttons.contains(Buttons::Down) {
                                println!("Down pressed");
                                joycon_states[0].forward = false;
                            } else if report.common.pushed_buttons.contains(Buttons::Up) {
                                println!("Down pressed");
                                joycon_states[0].forward = true;
                            } else if report.common.pushed_buttons.contains(Buttons::Left)
                                && report.common.pushed_buttons.contains(Buttons::Right)
                            {
                                println!("Left and right pressed - armed");
                                joycon_states[0].armed = true;
                            } else if report.common.pushed_buttons.contains(Buttons::SL)
                                && report.common.pushed_buttons.contains(Buttons::SR)
                            {
                                println!("L and R pressed - unarmed");
                                joycon_states[0].armed = false;
                            }

                            let armed = joycon_states[0].armed || joycon_states[1].armed;

                            let mut forward = true;
                            if joycon_states[0].armed && joycon_states[1].armed {
                                // both are armed, we go whatever diretion they agree on
                                if joycon_states[0].forward == joycon_states[1].forward {
                                    // both agree, we go whatever diretion they agree on
                                    forward = joycon_states[0].forward;
                                } else {
                                    // jesus, they disagree.  just go forward.
                                    forward = true;
                                }
                            } else if joycon_states[0].armed {
                                // use left joycon (0) to decide direction, it's armed
                                forward = joycon_states[0].forward;
                            } else if joycon_states[1].armed {
                                // use right joycon (1) to decide direction, it's armed
                                forward = joycon_states[1].forward;
                            }

                            println!(
                                "Horizontal: {}\t Vertical: {}\t forward: {}\t armed: {}\t joycon_states: {:?}",
                                horizontal_mapped, vertical_mapped, forward, armed, joycon_states
                            );

                            tx_clone
                                .send(CarCommand::SendData(
                                    horizontal_mapped,
                                    vertical_mapped,
                                    forward,
                                    armed,
                                ))
                                .unwrap();
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
