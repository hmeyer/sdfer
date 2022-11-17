use super::Primitive;
use anyhow::{bail, Result};
use std::collections::HashSet;

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
    fn static_code(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn expression(&self, p: &str) -> String {
        format!("length({}) - {:.8}", p, self.radius)
    }
}
