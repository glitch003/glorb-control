use crate::utils::map_range;

#[derive(Debug, Clone)]
pub struct JoyConState {
    pub forward: bool,
    pub armed: bool,
}

pub fn remap_left_joycon(horizontal: u16, vertical: u16) -> (u16, u16) {
    // horizontal min (left) 670
    // horizotal max (right) 3420
    let horizontal_mapped = map_range(horizontal, (670, 3240), (240, 1807), true);

    // vertical min (down) 1080
    // vertical max (up) 3240
    let vertical_mapped = map_range(vertical, (1080, 3240), (240, 1807), false);

    (horizontal_mapped, vertical_mapped)
}

pub fn remap_right_joycon(horizontal: u16, vertical: u16) -> (u16, u16) {
    // horizontal min (left) 700
    // horizotal max (right) 3600
    let horizontal_mapped = map_range(horizontal, (700, 3600), (240, 1807), true);

    // vertical min (down) 780
    // vertical max (up) 3000
    let vertical_mapped = map_range(vertical, (780, 3000), (240, 1807), false);

    (horizontal_mapped, vertical_mapped)
}
