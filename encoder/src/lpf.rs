use std::f32::consts::PI;

pub fn filter<T>(
    input: impl Iterator<Item = T>,
    cuttoff: f32,
    sample_rate: f32,
) -> impl Iterator<Item = f32>
where
    T: Into<f32>,
{
    let rc = 1.0 / (2.0 * PI * cuttoff);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (dt + rc);

    input.scan(127.0, move |carry, b| {
        *carry = *carry + (alpha * (b.into() - *carry));
        Some(*carry)
    })
}
