pub fn map_range(value: u16, from_range: (u16, u16), to_range: (u16, u16), invert: bool) -> u16 {
    let mut value = value;
    let (from_min, from_max) = from_range;
    let (to_min, to_max) = to_range;

    // If invert flag is set, swap the to_range values
    if invert {
        value = from_max + from_min - value;
    }

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
