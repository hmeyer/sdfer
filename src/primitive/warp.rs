use super::Primitive;
use anyhow::Result;
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
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        shared_code.push(
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
        self.primitive.expression(
            &format!("TwistXY({}, {:.8})", p, 2. * PI / self.height_per_rotation),
            shared_code,
        )
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        let rad_per_h = 2. * PI / self.height_per_rotation;
        let a = p[2] * rad_per_h;
        let sin_a = a.sin();
        let cos_a = a.cos();
        let rmat = na::Matrix2::new(cos_a, -sin_a, sin_a, cos_a);
        let r_xy = rmat * p.rows(0, 2);
        return self
            .primitive
            .eval(na::Vector3::new(r_xy[0], r_xy[1], p[2]));
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
    fn expression(&self, p: &str, shared_code: &mut Vec<String>) -> Result<String> {
        shared_code.push(
            r#"
vec3 BendAroundZ(vec3 p, float y_scale) {
    float a = atan(p.x, p.y);
    float r = length(p.xy);
    return vec3(a * y_scale, r, p.z);
}
"#
            .to_string(),
        );
        self.primitive.expression(
            &format!(
                "BendAroundZ({}, {:.8})",
                p,
                self.distance_for_full_circle * 0.5 / PI
            ),
            shared_code,
        )
    }
    fn eval(&self, p: na::Vector3<f32>) -> Result<f32> {
        let a = p[0].atan2(p[1]);
        let r = p.rows(0, 2).norm();
        let y_scale = self.distance_for_full_circle * 0.5 / PI;
        return self.primitive.eval(na::Vector3::new(a * y_scale, r, p[2]));
    }
}
