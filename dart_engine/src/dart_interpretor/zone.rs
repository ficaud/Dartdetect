const SECTOR_SPAN_DEGREES: f64 = 18.0;
const SECTOR_ORDER: [u8; 20] = [
    20, 1, 18, 4, 13, 6, 10, 15, 2, 17, 3, 19, 7, 16, 8, 11, 14, 9, 12, 5,
];

/// Resolve the numbered dartboard sector from an angle.
pub fn sector_from_angle(angle: f64) -> u8 {
    let normalized_angle = angle.rem_euclid(360.0);
    let sector_index = ((normalized_angle + (SECTOR_SPAN_DEGREES / 2.0)) / SECTOR_SPAN_DEGREES)
        .floor() as usize
        % SECTOR_ORDER.len();

    SECTOR_ORDER[sector_index]
}

#[cfg(test)]
mod tests {
    use super::sector_from_angle;

    #[test]
    fn quarter_turn_clockwise_maps_to_six() {
        assert_eq!(sector_from_angle(90.0), 6);
    }

    #[test]
    fn half_turn_maps_to_three() {
        assert_eq!(sector_from_angle(180.0), 3);
    }

    #[test]
    fn negative_angles_are_normalized() {
        assert_eq!(sector_from_angle(-18.0), 5);
    }
}
