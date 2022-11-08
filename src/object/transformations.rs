use super::{shader_mat3, shader_vec3};
use crate::object::Object;
use std::collections::HashSet;

pub struct Translate {
    object: Box<dyn Object>,
    vector: na::Vector3<f32>,
}

impl Translate {
    pub fn new(object: Box<dyn Object>, vector: na::Vector3<f32>) -> Translate {
        Translate { object, vector }
    }
}

impl Object for Translate {
    fn static_code(&self) -> HashSet<String> {
        self.object.static_code()
    }
    fn expression(&self, p: &str) -> String {
        self.object
            .expression(&format!("({}) - {}", p, shader_vec3(&self.vector)))
    }
}

pub struct Rotate {
    object: Box<dyn Object>,
    matrix: na::Matrix3<f32>,
}

impl Rotate {
    pub fn from_euler(object: Box<dyn Object>, r: f32, p: f32, y: f32) -> Rotate {
        Rotate {
            object,
            matrix: na::Matrix4::from_euler_angles(r, p, y)
                .fixed_slice::<3, 3>(0, 0)
                .into(),
        }
    }
}

impl Object for Rotate {
    fn static_code(&self) -> HashSet<String> {
        self.object.static_code()
    }
    fn expression(&self, p: &str) -> String {
        self.object
            .expression(&format!("{} * ({})", shader_mat3(&self.matrix), p))
    }
}

pub struct Scale {
    object: Box<dyn Object>,
    scale: na::Vector3<f32>,
}

impl Scale {
    pub fn new(object: Box<dyn Object>, scale: na::Vector3<f32>) -> Scale {
        Scale { object, scale }
    }
}

impl Object for Scale {
    fn static_code(&self) -> HashSet<String> {
        self.object.static_code()
    }
    fn expression(&self, p: &str) -> String {
        let d = self.object.expression(&format!(
            "({}) * {}",
            p,
            shader_vec3(&(na::Vector3::new(1., 1., 1.).component_div(&self.scale)))
        ));
        format!("({}) * {:.8}", d, self.scale.abs().min())
    }
}
