use super::{DistanceCalculation, Point, SensorPos};

/// Timing fake information to simulate pizeo sensors acquisition
#[derive(Debug)]
pub struct ImpactSimulator {
    pub t1: f64,
    pub t2: f64,
    pub t3: f64,
    pub t4: f64,
}

impl ImpactSimulator {
    /// Create a new sensor detection simulator.
    ///
    /// # Arguments
    /// - t1: Timing from sensor 1 in microseconds
    /// - t2: Timing from sensor 2 in microseconds
    /// - t3: Timing from sensor 3 in microseconds
    /// - t4: Timing from sensor 4 in microseconds
    pub fn new(t1: f64, t2: f64, t3: f64, t4: f64) -> Self {
        ImpactSimulator { t1, t2, t3, t4 }
    }

    /// Helper function to create a new `ImpactSimulator` from a given point of impact and sensor positions.
    ///
    /// # Arguments
    /// - impact: The point of impact to simulate
    /// - sensors: An array of sensor positions
    /// - baseline_time_us: The baseline time in microseconds to add to the calculated timings
    /// 
    /// This function calculates the timings for each sensor based on the distance from the impact point to each sensor
    pub fn from_point(impact: &Point, sensors: &[SensorPos; 4], baseline_time_us: f64) -> Self {
        let to_time = |sensor: &SensorPos| {
            let c = sensor.point();
            let distance_mm = ((impact.x - c.x).powi(2) + (impact.y - c.y).powi(2)).sqrt();
            baseline_time_us + (distance_mm * 1_000.0 / 343.0)
        };

        ImpactSimulator {
            t1: to_time(&sensors[0]),
            t2: to_time(&sensors[1]),
            t3: to_time(&sensors[2]),
            t4: to_time(&sensors[3]),
        }
    }
}

/// Implements the `DistanceCalculation` trait for `ImpactSimulator`, allowing it to calculate distances based on the simulated timings.
impl DistanceCalculation for ImpactSimulator {
    type Output = [f64; 4];

    /// Get the distances from the simulated timings using the speed of sound and the delta timings.
    ///
    /// # Arguments
    /// - self: The `ImpactSimulator` instance containing the simulated timings for each sensor.
    /// 
    /// This function return the four distances in millimeters corresponding to the timings of the sensors
    fn get_distances(&self, _point: Point) -> Self::Output {
        // First get the delta value from the timiings finding the minimum timing and then subtracting it from all the timings
        let min_time = self.t1.min(self.t2).min(self.t3).min(self.t4);
        let delta_t1 = self.t1 - min_time;
        let delta_t2 = self.t2 - min_time;
        let delta_t3 = self.t3 - min_time;
        let delta_t4 = self.t4 - min_time;

        // Now convert the delta timings to distances using the speed of sound (343 m/s)
        let speed_of_sound = 343.0; // in m/s
        let d1 = delta_t1 * speed_of_sound / 1_000.0; // get the distance in mm
        let d2 = delta_t2 * speed_of_sound / 1_000.0;
        let d3 = delta_t3 * speed_of_sound / 1_000.0;
        let d4 = delta_t4 * speed_of_sound / 1_000.0;

        // Return all the distance in an array
        [d1, d2, d3, d4]
    }
}
