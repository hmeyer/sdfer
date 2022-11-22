use super::{shader_mat3, shader_vec3, Primitive};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Translate {
    primitive: Box<dyn Primitive>,
    vector: na::Vector3<f32>,
}

impl Translate {
    pub fn new(primitive: Box<dyn Primitive>, vector: na::Vector3<f32>) -> Box<Translate> {
        Box::new(Translate { primitive, vector })
    }
}

impl Primitive for Translate {
    fn expression(&self, p: &str, shared_code: &mut HashSet<String>) -> String {
        self.primitive.expression(
            &format!("({}) - {}", p, shader_vec3(&self.vector)),
            shared_code,
        )
    }
}

#[derive(Clone)]
pub struct Rotate {
    primitive: Box<dyn Primitive>,
    matrix: na::Matrix3<f32>,
}

impl Rotate {
    pub fn from_euler(primitive: Box<dyn Primitive>, r: f32, p: f32, y: f32) -> Box<Rotate> {
        Box::new(Rotate {
            primitive,
            matrix: na::Matrix4::from_euler_angles(r, p, y)
                .fixed_slice::<3, 3>(0, 0)
                .into(),
        })
    }
}

impl Primitive for Rotate {
    fn expression(&self, p: &str, shared_code: &mut HashSet<String>) -> String {
        self.primitive.expression(
            &format!("{} * ({})", shader_mat3(&self.matrix), p),
            shared_code,
        )
    }
}

#[derive(Clone)]
pub struct Scale {
    primitive: Box<dyn Primitive>,
    scale: na::Vector3<f32>,
}

impl Scale {
    pub fn new(primitive: Box<dyn Primitive>, scale: na::Vector3<f32>) -> Box<Scale> {
        Box::new(Scale { primitive, scale })
    }
}

impl Primitive for Scale {
    fn expression(&self, p: &str, shared_code: &mut HashSet<String>) -> String {
        let d = self.primitive.expression(
            &format!(
                "({}) * {}",
                p,
                shader_vec3(&(na::Vector3::new(1., 1., 1.).component_div(&self.scale)))
            ),
            shared_code,
        );
        format!("({}) * {:.8}", d, self.scale.abs().min())
    }
}
