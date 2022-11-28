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

fn v3_round(v: na::Vector3<f32>) -> na::Vector3<f32> {
    na::Vector3::new(v[0].round(), v[1].round(), v[2].round())
}

fn v3_clamp(v: na::Vector3<f32>, min: na::Vector3<f32>, max: na::Vector3<f32>) -> na::Vector3<f32> {
    na::Vector3::new(
        v[0].clamp(min[0], max[0]),
        v[1].clamp(min[1], max[1]),
        v[2].clamp(min[2], max[2]),
    )
}

impl Primitive for Repeat {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
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
    fn eval(&self, p: na::Vector3<f32>) -> f32 {
        let rp = v3_round(p).component_div(&self.bounds);
        let rp = v3_clamp(rp, self.repeats_min, self.repeats_max);
        let p = p - self.bounds.component_mul(&rp);
        self.primitive.eval(p)
    }
}
