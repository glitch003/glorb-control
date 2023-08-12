use joycon_rs::prelude::*;

use crate::joycons::JoyConState;

#[derive(Debug, Clone)]
pub struct StateManager {
    pub l: JoyConState,
    pub r: JoyConState,
}

impl StateManager {
    pub fn new() -> StateManager {
        StateManager {
            l: JoyConState {
                forward: true,
                armed: false,
            },
            r: JoyConState {
                forward: true,
                armed: false,
            },
        }
    }

    pub fn set_state(&mut self, joycon: JoyConDeviceType, forward: bool, armed: bool) {
        match joycon {
            JoyConDeviceType::JoyConL => {
                self.l.forward = forward;
                self.l.armed = armed;
            }
            JoyConDeviceType::JoyConR => {
                self.r.forward = forward;
                self.r.armed = armed;
            }
            _ => {
                println!("Unknown JoyCon type {:?}", joycon)
            }
        }
    }
}
