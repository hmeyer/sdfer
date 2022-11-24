use super::{shader_vec3, Primitive};
use anyhow::{bail, Result};

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
    fn expression(&self, p: &str, _shared_code: &mut Vec<String>) -> Result<String> {
        Ok(format!(
            "dot({}, {}) + {:.8}",
            p,
            shader_vec3(&self.normal),
            self.d
        ))
    }
}
