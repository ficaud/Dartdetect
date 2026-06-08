#[derive(Debug, PartialEq)]
enum MultiplierKind {
    Zero,
    Single,
    Double,
    Triple,
    BlackBorder,
}

const DOUBLE_BULL_RADIUS: f32 = 6.35;
const SINGLE_BULL_RADIUS: f32 = 15.9;
const TRIPLE_INNER_RADIUS: f32 = 97.0;
const TRIPLE_OUTER_RADIUS: f32 = 105.0;
const DOUBLE_INNER_RADIUS: f32 = 162.0;
const DOUBLE_OUTER_RADIUS: f32 = 170.0;
const BLACK_BORDER_RADIUS: f32 = 225.0;

/// Convert a radius value to a `MultiplierKind` enum variant.
///
/// # Arguments
/// - radius: A `f32` value representing the distance from the center of the dartboard.
///
/// This function determines the `MultiplierKind` based on the dartboard's scoring zones,
/// such as single, double, and triple areas, as well as the bullseye and black border zones.
fn multiplier_kind_from_radius(radius: f32) -> MultiplierKind {
    if radius <= DOUBLE_BULL_RADIUS {
        MultiplierKind::Double
    } else if radius <= SINGLE_BULL_RADIUS {
        MultiplierKind::Single
    } else if radius <= TRIPLE_INNER_RADIUS {
        MultiplierKind::Single
    } else if radius <= TRIPLE_OUTER_RADIUS {
        MultiplierKind::Triple
    } else if radius <= DOUBLE_INNER_RADIUS {
        MultiplierKind::Single
    } else if radius <= DOUBLE_OUTER_RADIUS {
        MultiplierKind::Double
    } else if radius <= BLACK_BORDER_RADIUS {
        MultiplierKind::BlackBorder
    } else {
        MultiplierKind::Zero
    }
}

/// Convert a radius value to a score multiplier.
///
/// This function takes a radius value (distance from the center of the dartboard) and returns the corresponding score multiplier.
/// The multiplier is determined based on the dartboard's scoring zones, such as single, double, and triple areas.
pub fn multiplier_from_radius(radius: f32) -> u32 {
    match multiplier_kind_from_radius(radius) {
        MultiplierKind::Zero | MultiplierKind::BlackBorder => 0,
        MultiplierKind::Single => 1,
        MultiplierKind::Double => 2,
        MultiplierKind::Triple => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::multiplier_from_radius;

    #[test]
    fn double_bull_radius_maps_to_double() {
        assert_eq!(multiplier_from_radius(6.35), 2);
    }

    #[test]
    fn single_bull_radius_maps_to_single() {
        assert_eq!(multiplier_from_radius(10.0), 1);
    }

    #[test]
    fn triple_ring_radius_maps_to_triple() {
        assert_eq!(multiplier_from_radius(100.0), 3);
    }

    #[test]
    fn outer_double_ring_maps_to_double() {
        assert_eq!(multiplier_from_radius(165.0), 2);
    }

    #[test]
    fn black_border_radius_maps_to_black_border() {
        assert_eq!(multiplier_from_radius(200.0), 0);
    }

    #[test]
    fn outside_board_maps_to_zero() {
        assert_eq!(multiplier_from_radius(230.0), 0);
    }
}
