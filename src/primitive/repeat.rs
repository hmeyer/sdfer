use super::{shader_vec3, Primitive};
use anyhow::{bail, Result};

#[derive(Clone)]
pub struct Repeat {
    primitive: Box<dyn Primitive>,
    bounds: na::Vector3<f32>,
    repeats_min: na::Vector3<f32>,
    repeats_max: na::Vector3<f32>,
}

impl Repeat {
    pub fn new(
        primitive: Box<dyn Primitive>,
        bounds: na::Vector3<f32>,
        repeats_min: na::Vector3<i32>,
        repeats_max: na::Vector3<i32>,
    ) -> Result<Box<Repeat>> {
        if bounds.min() <= 0.0 {
            bail!("bound must be larger 0 (was {}).", bounds);
        }
        if (repeats_max - repeats_min).min() < 0 {
            bail!(
                "repeats range must non-negative (was {:?} - {:?}).",
                repeats_min,
                repeats_max
            );
        }
        Ok(Box::new(Repeat {
            primitive,
            bounds,
            repeats_min: repeats_min.cast::<f32>(),
            repeats_max: repeats_max.cast::<f32>(),
        }))
    }
}

impl Primitive for Repeat {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> String {
        self.primitive.expression(
            &format!(
                "{p} - {bounds} * clamp(round({p} / {bounds}), {rep_min}, {rep_max})",
                p = p,
                bounds = shader_vec3(&self.bounds),
                rep_min = shader_vec3(&(self.repeats_min)),
                rep_max = shader_vec3(&(self.repeats_max))
            ),
            shared_code,
        )
    }
}
