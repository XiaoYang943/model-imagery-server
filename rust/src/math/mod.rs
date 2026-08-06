mod cartesian3;
mod epsilon;
mod to_radians;
pub use cartesian3::*;
pub use epsilon::*;
pub use to_radians::*;
pub fn equals_epsilon(
    left: f64,
    right: f64,
    relative_epsilon: Option<f64>,
    absolute_epsilon: Option<f64>,
) -> bool {
    let relative_epsilon = relative_epsilon.unwrap_or(0.0);
    let absolute_epsilon = absolute_epsilon.unwrap_or(relative_epsilon);
    let diff = (left - right).abs();
    return diff <= absolute_epsilon || diff <= relative_epsilon * left.abs();
}
