use super::Primitive;
use anyhow::{bail, Result};

#[derive(Clone)]
pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Result<Box<dyn Primitive>> {
        if radius <= 0. {
            bail!("radius should be positive (was {}).", radius);
        }
        Ok(Box::new(Sphere { radius }))
    }
}

impl Primitive for Sphere {
    fn expression(&self, p: &str, _shared_code: &mut Vec<String>) -> Result<String> {
        Ok(format!("length({}) - {:.8}", p, self.radius))
    }
    fn eval(&self, p: na::Vector3<f32>) -> f32 {
        p.norm() - self.radius
    }
}
