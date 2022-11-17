use crate::primitive::Primitive;
use std::collections::HashSet;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Twist {
    primitive: Box<dyn Primitive>,
    height_per_rotation: f32,
}

impl Twist {
    pub fn new(primitive: Box<dyn Primitive>, height_per_rotation: f32) -> Box<Twist> {
        Box::new(Twist {
            primitive,
            height_per_rotation,
        })
    }
}

impl Primitive for Twist {
    fn static_code(&self) -> HashSet<String> {
        self.primitive.static_code()
    }
    fn expression(&self, p: &str) -> String {
        let a = format!("({}).z * {:.8}", p, 2. * PI / self.height_per_rotation);
        self.primitive.expression(&format!(
            "vec3(mat2(cos({a}), -sin({a}), sin({a}), cos({a})) * ({p}).xy, ({p}).z)",
            a = a,
            p = p
        ))
    }
}
