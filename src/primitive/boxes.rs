use super::{shader_vec3, Primitive};
use anyhow::{bail, Result};
use std::collections::HashSet;

#[derive(Clone)]
pub struct ExactBox {
    size: na::Vector3<f32>,
}

impl ExactBox {
    pub fn new(size: na::Vector3<f32>) -> Result<Box<dyn Primitive>> {
        if size.min() <= 0. {
            bail!("all dimensions must be greater zero (was {}).", size);
        }
        Ok(Box::new(ExactBox { size: size / 2. }))
    }
}

impl Primitive for ExactBox {
    fn static_code(&self) -> HashSet<String> {
        HashSet::from([r#"
float Box( vec3 p, vec3 b ) {
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}
"#
        .to_string()])
    }
    fn expression(&self, p: &str) -> String {
        format!("Box({}, {})", p, shader_vec3(&self.size))
    }
}

#[derive(Clone)]
pub struct RoundBox {
    size: na::Vector3<f32>,
    radius: f32,
}

impl RoundBox {
    pub fn new(size: na::Vector3<f32>, radius: f32) -> Result<Box<dyn Primitive>> {
        if size.min() <= 0. {
            bail!("all dimensions must be greater zero (was {}).", size);
        }
        if radius < 0. {
            bail!("radius must be greater equal zero (was {}).", radius);
        }
        Ok(Box::new(RoundBox {
            size: size / 2.0 - na::Vector3::new(radius, radius, radius),
            radius,
        }))
    }
}

impl Primitive for RoundBox {
    fn static_code(&self) -> HashSet<String> {
        HashSet::from([r#"
float RoundBox( vec3 p, vec3 b, float r ) {
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0) - r;
}
"#
        .to_string()])
    }
    fn expression(&self, p: &str) -> String {
        format!(
            "RoundBox({}, {}, {:.8})",
            p,
            shader_vec3(&self.size),
            self.radius
        )
    }
}
