/// Defines the `DistanceCalculation` trait, which provides a method to calculate distances
///
/// This trait is used by various structs to implement the logic for calculating distances based on 
/// different inputs, such as sensor timings or guessed points.
pub trait DistanceCalculation {
    type Output;

    fn get_distances(&self) -> Self::Output;
}
