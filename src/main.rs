use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use arc_swap::ArcSwap;
use joycon_rs::prelude::*;

// mod sbus_parser;
// use sbus_parser::SBusPacketParser;

mod sbus_writer;

mod car;
#[cfg(feature = "car")]
use car::Car;
use car::CarCommand;

mod utils;
use utils::mix_joycon_states;

mod joycons;
use joycons::{remap_left_joycon, remap_right_joycon};

mod state_manager;
use state_manager::StateManager;

fn main() {
    // Create a channel for sending commands
    let (car_tx, car_rx) = mpsc::channel();

    //  Spawn a dedicated thread that owns `car`
    let _car_handle = thread::spawn(move || {
        #[cfg(feature = "car")]
        let mut car = Car::new();
        for command in car_rx {
            println!("Sending command to car: {:?}", command);
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

    let state_store = Arc::new(ArcSwap::from(Arc::new(StateManager::new())));

    managed_devices
        .into_iter()
        .chain(new_devices)
        .flat_map(|dev| SimpleJoyConDriver::new(&dev))
        .try_for_each::<_, JoyConResult<()>>(|driver| {
            let device_type = driver.joycon().device_type();

            println!("Found JoyCon: {:?}", device_type);

            let car_tx_clone = car_tx.clone();
            let state_store = state_store.clone();

            // Change JoyCon to Simple hid mode.
            // let simple_hid_mode = SimpleHIDMode::new(driver)?;
            let standard_full_mode = StandardFullMode::new(driver)?;

            let device_type = Arc::new(device_type);

            // Spawn thread
            thread::spawn(move || {
                loop {
                    // Forward the report to the car thread
                    let report = standard_full_mode.read_input_report();

                    match report {
                        Ok(report) => {
                            // println!("{:?}", report);
                            let state_arc = state_store.load_full();
                            let mut state = (*state_arc).clone();

                            match *device_type {
                                JoyConDeviceType::JoyConL => {
                                    // left joycon buttons

                                    if report.common.pushed_buttons.contains(Buttons::Down) {
                                        println!("Down pressed");
                                        state.l.forward = false;
                                    } else if report.common.pushed_buttons.contains(Buttons::Up) {
                                        println!("Up pressed");
                                        state.l.forward = true;
                                    } else if report.common.pushed_buttons.contains(Buttons::Left)
                                        && report.common.pushed_buttons.contains(Buttons::Right)
                                    {
                                        println!("Left and right pressed - armed");
                                        state.l.armed = true;
                                    } else if report.common.pushed_buttons.contains(Buttons::SL)
                                        && report.common.pushed_buttons.contains(Buttons::SR)
                                    {
                                        println!("L and R pressed - unarmed");
                                        state.l.armed = false;
                                    }

                                    let (forward, armed) = mix_joycon_states(&state);

                                    let (horizontal_mapped, vertical_mapped) = remap_left_joycon(
                                        report.common.left_analog_stick_data.horizontal,
                                        report.common.left_analog_stick_data.vertical,
                                        state.l.forward,
                                    );

                                    if state.l.armed && !state.r.armed {
                                        car_tx_clone
                                            .send(CarCommand::SendData(
                                                horizontal_mapped,
                                                vertical_mapped,
                                                forward,
                                                armed,
                                            ))
                                            .unwrap();
                                    }

                                    state.set_state(
                                        (*device_type).clone(),
                                        state.l.forward,
                                        state.l.armed,
                                    );
                                }
                                JoyConDeviceType::JoyConR => {
                                    // right joycon buttons
                                    if report.common.pushed_buttons.contains(Buttons::B) {
                                        println!("B pressed");
                                        state.r.forward = false;
                                    } else if report.common.pushed_buttons.contains(Buttons::X) {
                                        println!("X pressed");
                                        state.r.forward = true;
                                    } else if report.common.pushed_buttons.contains(Buttons::Y)
                                        && report.common.pushed_buttons.contains(Buttons::A)
                                    {
                                        println!("Y and A pressed - armed");
                                        state.r.armed = true;
                                    } else if report.common.pushed_buttons.contains(Buttons::SL)
                                        && report.common.pushed_buttons.contains(Buttons::SR)
                                    {
                                        println!("L and R pressed - unarmed");
                                        state.r.armed = false;
                                    }

                                    let (horizontal_mapped, vertical_mapped) = remap_right_joycon(
                                        report.common.right_analog_stick_data.horizontal,
                                        report.common.right_analog_stick_data.vertical,
                                        state.r.forward,
                                    );

                                    // println!(
                                    //     "Left joycon sending Horizontal: {}\t Vertical: {}\t forward: {}\t armed: {}\t joycon_states: {:?}",
                                    //     horizontal_mapped, vertical_mapped, forward, armed, state
                                    // );
                                    let (forward, armed) = mix_joycon_states(&state);

                                    if state.r.armed && !state.l.armed {
                                        car_tx_clone
                                            .send(CarCommand::SendData(
                                                horizontal_mapped,
                                                vertical_mapped,
                                                forward,
                                                armed,
                                            ))
                                            .unwrap();
                                    }

                                    state.set_state(
                                        (*device_type).clone(),
                                        state.r.forward,
                                        state.r.armed,
                                    );
                                }
                                _ => {
                                    println!("Unknown JoyCon type {:?}", device_type)
                                }
                            };

                            state_store.store(Arc::new(state));
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
