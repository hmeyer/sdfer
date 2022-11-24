use super::Primitive;
use anyhow::{bail, Result};
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Torus {
    inner: f32,
    outer: f32,
    cap_angle: Option<f32>,
}

impl Torus {
    pub fn new(inner: f32, outer: f32) -> Result<Box<dyn Primitive>> {
        Torus::new_impl(inner, outer, None)
    }
    pub fn new_capped(inner: f32, outer: f32, cap_angle: f32) -> Result<Box<dyn Primitive>> {
        Torus::new_impl(inner, outer, Some(cap_angle))
    }
    fn new_impl(inner: f32, outer: f32, cap_angle: Option<f32>) -> Result<Box<dyn Primitive>> {
        if inner >= outer {
            bail!("inner radius must be smaller than outer radius.")
        }
        if outer <= 0.0 {
            bail!("outer radius must be greater than 0.")
        }
        match cap_angle {
            Some(a) if a < 0. => bail!("cap_angle must be positive (was {})!", a),
            Some(a) if a > PI => {
                bail!("cap_angle must be less equal than pi (was {})!", a)
            }
            _ => {}
        }
        Ok(Box::new(Torus {
            inner,
            outer,
            cap_angle,
        }))
    }
}

impl Primitive for Torus {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        match self.cap_angle {
            None => {
                shared_code.push(
                    r#"
float Torus(vec3 p, vec2 t) {
    vec2 q = vec2(length(p.xy) - t.x, p.z);
    return length(q) - t.y;
}"#
                    .to_string(),
                );
                Ok(format!(
                    "Torus({}, vec2({:.8}, {:.8}))",
                    p,
                    (self.inner + self.outer) / 2.0,
                    (self.outer - self.inner) / 2.0
                ))
            }
            Some(a) => {
                shared_code.push(
                    r#"
float CappedTorus(vec3 p, float ra, float rb, vec2 an) {
    p.x = abs(p.x);
    float k = (an.y * p.x > an.x * p.y) ? dot(p.xy, an) : length(p.xy);
    return sqrt( dot(p, p) + ra * ra - 2.0 * ra * k) - rb;
}
"#
                    .to_string(),
                );
                Ok(format!(
                    "CappedTorus({}, {:.8}, {:.8}, vec2({:.8}, {:.8}))",
                    p,
                    (self.inner + self.outer) / 2.0,
                    (self.outer - self.inner) / 2.0,
                    a.sin(),
                    a.cos()
                ))
            }
        }
    }
}
