use super::{shader_vec3, Primitive};
use anyhow::{bail, Result};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Plane {
    normal: na::Vector3<f32>,
    d: f32,
}

impl Plane {
    pub fn new(normal: na::Vector3<f32>, d: f32) -> Result<Box<dyn Primitive>> {
        if let Some(normal) = normal.try_normalize(0.) {
            return Ok(Box::new(Plane { normal, d }));
        }
        bail!("Normal must be != 0.");
    }
}

impl Primitive for Plane {
    fn static_code(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn expression(&self, p: &str) -> String {
        format!("dot({}, {}) + {:.8}", p, shader_vec3(&self.normal), self.d)
    }
}
