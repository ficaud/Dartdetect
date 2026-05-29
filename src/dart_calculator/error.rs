/// Provides error calculation for a source.
///
/// Implementors may return absolute or relative errors depending on
/// the acquisition model.
pub trait ErrorCalculation {
    type Output;

    fn calculate_error(&self, actual_distances: &[f64; 4]) -> Self::Output;
}