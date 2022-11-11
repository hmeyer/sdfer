use super::{shader_mat3, shader_vec3};
use crate::primitive::Primitive;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Translate {
    primitive: Box<dyn Primitive>,
    vector: na::Vector3<f32>,
}

impl Translate {
    pub fn new(primitive: Box<dyn Primitive>, vector: na::Vector3<f32>) -> Translate {
        Translate { primitive, vector }
    }
}

impl Primitive for Translate {
    fn static_code(&self) -> HashSet<String> {
        self.primitive.static_code()
    }
    fn expression(&self, p: &str) -> String {
        self.primitive
            .expression(&format!("({}) - {}", p, shader_vec3(&self.vector)))
    }
}

#[derive(Clone)]
pub struct Rotate {
    primitive: Box<dyn Primitive>,
    matrix: na::Matrix3<f32>,
}

impl Rotate {
    pub fn from_euler(primitive: Box<dyn Primitive>, r: f32, p: f32, y: f32) -> Rotate {
        Rotate {
            primitive,
            matrix: na::Matrix4::from_euler_angles(r, p, y)
                .fixed_slice::<3, 3>(0, 0)
                .into(),
        }
    }
}

impl Primitive for Rotate {
    fn static_code(&self) -> HashSet<String> {
        self.primitive.static_code()
    }
    fn expression(&self, p: &str) -> String {
        self.primitive
            .expression(&format!("{} * ({})", shader_mat3(&self.matrix), p))
    }
}

#[derive(Clone)]
pub struct Scale {
    primitive: Box<dyn Primitive>,
    scale: na::Vector3<f32>,
}

impl Scale {
    pub fn new(primitive: Box<dyn Primitive>, scale: na::Vector3<f32>) -> Scale {
        Scale { primitive, scale }
    }
}

impl Primitive for Scale {
    fn static_code(&self) -> HashSet<String> {
        self.primitive.static_code()
    }
    fn expression(&self, p: &str) -> String {
        let d = self.primitive.expression(&format!(
            "({}) * {}",
            p,
            shader_vec3(&(na::Vector3::new(1., 1., 1.).component_div(&self.scale)))
        ));
        format!("({}) * {:.8}", d, self.scale.abs().min())
    }
}
