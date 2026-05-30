/// A 2D point in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// An enum representing the positions of the four sensors, each containing a `Point` that defines its location.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorPos {
    C1(Point),
    C2(Point),
    C3(Point),
    C4(Point),
}

/// Sensor position and point struct definitions, along with helper methods to access the point data from the sensor positions.
impl SensorPos {
    /// Helper method to retrieve the `Point` associated with a given `SensorPos` variant.
    ///
    /// This method allows for easy access to the coordinates of the sensor positions when calculating distances or simulating impacts.
    pub(crate) fn point(&self) -> &Point {
        match self {
            SensorPos::C1(point)
            | SensorPos::C2(point)
            | SensorPos::C3(point)
            | SensorPos::C4(point) => point,
        }
    }
}

/// Point new constructor implementation
impl Point {
    /// Create a new point.
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}