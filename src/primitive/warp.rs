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

#[derive(Clone)]
pub struct Bend {
    primitive: Box<dyn Primitive>,
    distance_for_full_circle: f32,
}

impl Bend {
    pub fn new(primitive: Box<dyn Primitive>, distance_for_full_circle: f32) -> Box<Bend> {
        Box::new(Bend {
            primitive,
            distance_for_full_circle,
        })
    }
}

impl Primitive for Bend {
    fn static_code(&self) -> HashSet<String> {
        let mut code_set = self.primitive.static_code();
        code_set.insert(
            r#"
vec3 BendAroundZ(vec3 p, float y_scale) {
    float a = atan(p.x, p.y);
    float r = length(p.xy);
    return vec3(a * y_scale, r, p.z);
}
"#
            .to_string(),
        );
        code_set
    }
    fn expression(&self, p: &str) -> String {
        self.primitive.expression(&format!(
            "BendAroundZ({}, {:.8})",
            p,
            self.distance_for_full_circle * 0.5 / PI
        ))
    }
}
