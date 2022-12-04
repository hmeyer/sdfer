use super::{shader_mat3, shader_vec3, Primitive};
use anyhow::Result;

#[derive(Clone)]
pub struct Translate {
    primitive: Box<dyn Primitive>,
    vector: glm::Vec3,
}

impl Translate {
    pub fn new(primitive: Box<dyn Primitive>, vector: glm::Vec3) -> Box<Translate> {
        Box::new(Translate { primitive, vector })
    }
}

impl Primitive for Translate {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        self.primitive.expression(
            &format!("({}) - {}", p, shader_vec3(&self.vector)),
            shared_code,
        )
    }
    fn eval(&self, p: glm::Vec3) -> f32 {
        self.primitive.eval(p - self.vector)
    }
}

#[derive(Clone)]
pub struct Rotate {
    primitive: Box<dyn Primitive>,
    matrix: glm::Mat3x3,
}

impl Rotate {
    pub fn from_euler(primitive: Box<dyn Primitive>, r: f32, p: f32, y: f32) -> Box<Rotate> {
        let mat = glm::identity::<f32, 4>();
        let mat = glm::rotate_x(&mat, p);
        let mat = glm::rotate_y(&mat, y);
        let mat = glm::rotate_z(&mat, r);
        Box::new(Rotate {
            primitive,
            matrix: mat.fixed_slice::<3, 3>(0, 0).into(),
        })
    }
}

impl Primitive for Rotate {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        self.primitive.expression(
            &format!("{} * ({})", shader_mat3(&self.matrix), p),
            shared_code,
        )
    }
    fn eval(&self, p: glm::Vec3) -> f32 {
        self.primitive.eval(self.matrix * p)
    }
}

#[derive(Clone)]
pub struct Scale {
    primitive: Box<dyn Primitive>,
    scale: glm::Vec3,
}

impl Scale {
    pub fn new(primitive: Box<dyn Primitive>, scale: glm::Vec3) -> Box<Scale> {
        Box::new(Scale { primitive, scale })
    }
}

impl Primitive for Scale {
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        let d = self.primitive.expression(
            &format!(
                "({}) * {}",
                p,
                shader_vec3(&(glm::vec3(1., 1., 1.).component_div(&self.scale)))
            ),
            shared_code,
        )?;
        Ok(format!("({}) * {:.8}", d, self.scale.abs().min()))
    }
    fn eval(&self, p: glm::Vec3) -> f32 {
        self.primitive.eval(p.component_mul(&self.scale))
    }
}
