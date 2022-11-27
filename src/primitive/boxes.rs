use super::{shader_vec3, Primitive};
use anyhow::{bail, Result};

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
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        shared_code.push(
            r#"
float Box( vec3 p, vec3 b ) {
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}
"#
            .to_string(),
        );
        Ok(format!("Box({}, {})", p, shader_vec3(&self.size)))
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        let q = v3_abs(p) - self.size;
        Ok(
            q.sup(&na::Vector3::new(0_f32, 0_f32, 0_f32)).norm()
                + q[1].max(q[2]).max(q[0]).min(0.0),
        )
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
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        shared_code.push(
            r#"
float RoundBox( vec3 p, vec3 b, float r ) {
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0) - r;
}
"#
            .to_string(),
        );
        Ok(format!(
            "RoundBox({}, {}, {:.8})",
            p,
            shader_vec3(&self.size),
            self.radius
        ))
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        let q = v3_abs(p) - self.size;
        Ok(
            q.sup(&na::Vector3::new(0_f32, 0_f32, 0_f32)).norm()
                + q[1].max(q[2]).max(q[0]).min(0.0)
                - self.radius,
        )
    }
}

fn v3_abs(v: na::Vector3<f32>) -> na::Vector3<f32> {
    na::Vector3::new(v[0].abs(), v[1].abs(), v[2].abs())
}
