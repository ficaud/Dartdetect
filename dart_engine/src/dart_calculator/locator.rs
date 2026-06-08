use super::solver::Solver;
use super::{Point, SensorPos};

/// Reusable entry point for locating impacts with a fixed sensor layout.
pub struct ImpactLocator {
    pub sensors: [SensorPos; 4],
    pub timings_us: [f64; 4],
}

/// ImpactLocator implementation, which provides a method to locate the impact point based on the sensor timings and positions.
impl ImpactLocator {
    /// Create a new `ImpactLocator` with the given sensor positions and timings.
    ///
    /// # Arguments
    /// - sensors: An array of `SensorPos` representing the positions of the four sensors.
    /// - timings_us: An array of `f64` representing the timings in microseconds.
    ///
    /// This function initializes the `ImpactLocator` with the provided sensor positions and timings, allowing it to later calculate the impact point using the `locate` method.
    pub fn new(sensors: [SensorPos; 4], timings_us: [f64; 4]) -> Self {
        ImpactLocator {
            sensors,
            timings_us,
        }
    }

    /// Locate the impact point based on the sensor position and impact timings.
    ///
    /// # Arguments
    /// - self: The `ImpactLocator` instance containing the sensor positions and timings.
    ///
    /// This function uses the `Solver` to calculate the impact point based on the provided sensor positions and impact timings.
    pub fn locate(&self) -> Option<Point> {
        Solver::new_from_timings(self.sensors, self.timings_us).solve()
    }
}

#[cfg(test)]
mod tests {
    use super::ImpactLocator;
    use crate::dart_calculator::{ImpactSimulator, Point, SensorPos};

    #[test]
    fn iterative_guess_loop_reaches_impact_from_simulator() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let sensors = [
            SensorPos::C1(Point::new(0.0, 500.0)),
            SensorPos::C2(Point::new(500.0, 500.0)),
            SensorPos::C3(Point::new(500.0, 0.0)),
            SensorPos::C4(Point::new(0.0, 0.0)),
        ];

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos() as u64;
        let impact_x = (seed % 501) as f64;
        let impact_y = ((seed / 997) % 501) as f64;
        let impact_point = Point::new(impact_x, impact_y);
        println!(
            "Target impact point: ({}, {})",
            impact_point.x as u32, impact_point.y as u32
        );

        let baseline_time_us = 1_000.0;
        let impact_simulator =
            ImpactSimulator::from_point(&impact_point, &sensors, baseline_time_us);
        let locator = ImpactLocator::new(
            sensors,
            [
                impact_simulator.t1,
                impact_simulator.t2,
                impact_simulator.t3,
                impact_simulator.t4,
            ],
        );
        let reached_impact = locator.locate();

        println!("Reached impact point: {:?}", reached_impact);

        match reached_impact {
            Some(point) => {
                assert_eq!(point.x, impact_point.x);
                assert_eq!(point.y, impact_point.y);
            }
            None => panic!("solver did not find an impact point"),
        }
    }
}
