use super::Primitive;
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
        let mut code_set = self.primitive.static_code();
        code_set.insert(
            r#"
vec3 TwistXY(vec3 p, float rad_per_h) {
    float a = p.z * rad_per_h;
    float sin_a = sin(a);
    float cos_a = cos(a);
    mat2 rmat = mat2(cos_a, -sin_a, sin_a, cos_a);
    return vec3(rmat * p.xy, p.z);
}
"#
            .to_string(),
        );
        code_set
    }
    fn expression(&self, p: &str) -> String {
        self.primitive.expression(&format!(
            "TwistXY({}, {:.8})",
            p,
            2. * PI / self.height_per_rotation
        ))
    }
}
